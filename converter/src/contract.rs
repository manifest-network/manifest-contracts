use crate::consts::{CONTRACT_NAME, CONTRACT_VERSION};
use crate::error::AmountError::NonPayable;
use crate::error::ConfigError::SameDenom;
use crate::error::ContractError;
use crate::error::MigrateError::InvalidContractName;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, ADMIN, CONFIG};
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, MigrateInfo, Response, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use cw_utils::nonpayable;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    nonpayable(&info).map_err(|_| ContractError::AmountError(NonPayable))?;
    let admin = deps.api.addr_validate(msg.admin.as_str())?;

    // Rate is validated in its constructor
    // Denoms are validated in their constructors

    let config = Config {
        poa_admin: deps.api.addr_validate(msg.poa_admin.as_str())?,
        rate: crate::rate::Rate::parse(&msg.rate)?,
        source_denom: crate::denom::Denom::new(msg.source_denom)?,
        target_denom: crate::denom::Denom::new(msg.target_denom)?,
        paused: msg.paused,
    };

    config.validate()?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    ADMIN.set(deps, Some(admin))?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Config {} => query::config(deps),
        Admin {} => query::admin(deps),
    }
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;
    match msg {
        UpdateAdmin { admin } => exec::update_admin(deps, info, admin),
        UpdateConfig { config } => exec::update_config(deps, info, config),
        Convert {} => exec::convert(deps.as_ref(), env, info),
    }
}

pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
    _info: MigrateInfo,
) -> Result<Response, ContractError> {
    let stored = get_contract_version(deps.storage)?;

    if stored.contract != CONTRACT_NAME {
        return Err(ContractError::MigrateError(InvalidContractName));
    }

    if stored.version == CONTRACT_VERSION {
        return Ok(Response::new()
            .add_attribute("action", "migrate")
            .add_attribute("note", "already at latest version")
            .add_attribute("version", CONTRACT_VERSION));
    }

    // TODO: Add migration steps when needed

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("contract", CONTRACT_NAME)
        .add_attribute("from_version", stored.version)
        .add_attribute("to_version", CONTRACT_VERSION))
}

mod query {
    use super::*;

    pub fn config(deps: Deps) -> StdResult<Binary> {
        to_json_binary(&CONFIG.load(deps.storage)?)
    }

    pub fn admin(deps: Deps) -> StdResult<Binary> {
        to_json_binary(&ADMIN.query_admin(deps)?)
    }
}

mod exec {
    use super::*;
    use crate::denom::Denom;
    use crate::error::AdminError::{CannotRenounce, NotAdmin};
    use crate::error::ConvertError::{InvalidFunds, InvalidSourceDenom};
    use crate::msg::UpdateConfig;
    use crate::rate::Rate;
    use cosmwasm_std::{AnyMsg, BankMsg, CosmosMsg};
    use cw_utils::one_coin;
    use manifest_std::cosmos::authz::v1beta1::MsgExec;
    use manifest_std::google::protobuf::Any;
    use manifest_std::liftedinit::manifest::v1::MsgBurnHeldBalance;
    use manifest_std::osmosis::tokenfactory::v1beta1::MsgMint;
    use prost::Message;

    pub fn update_admin(
        deps: DepsMut,
        info: MessageInfo,
        admin: Option<String>,
    ) -> Result<Response, ContractError> {
        nonpayable(&info).map_err(|_| ContractError::AmountError(NonPayable))?;
        ADMIN
            .assert_admin(deps.as_ref(), &info.sender)
            .map_err(|_| ContractError::AdminError(NotAdmin))?;

        let admin_str = admin.ok_or(ContractError::AdminError(CannotRenounce))?;
        let new = deps.api.addr_validate(&admin_str)?;

        let res = ADMIN
            .execute_update_admin(deps, info, Some(new))
            .map_err(|_| ContractError::AdminError(NotAdmin))?;
        Ok(res
            .add_attribute("action", "update_admin")
            .add_attribute("contract", CONTRACT_NAME)
            .add_attribute("version", CONTRACT_VERSION)
            .add_attribute("new_admin", admin_str))
    }

    // Update the contract configuration with new values
    pub fn update_config(
        deps: DepsMut,
        info: MessageInfo,
        config: UpdateConfig,
    ) -> Result<Response, ContractError> {
        nonpayable(&info).map_err(|_| ContractError::AmountError(NonPayable))?;
        ADMIN
            .assert_admin(deps.as_ref(), &info.sender)
            .map_err(|_| ContractError::AdminError(NotAdmin))?;

        if config.is_empty() {
            return Ok(Response::new()
                .add_attribute("action", "update_config")
                .add_attribute("note", "empty config, no changes made"));
        }
        let mut current_config = CONFIG.load(deps.storage)?;

        if config.is_noop(&current_config) {
            return Ok(Response::new()
                .add_attribute("action", "update_config")
                .add_attribute("note", "identical config, no changes made"));
        }

        if let Some(poa_admin) = config.poa_admin {
            let poa_admin_addr = deps.api.addr_validate(&poa_admin)?;
            current_config.poa_admin = poa_admin_addr;
        }

        if let Some(rate) = config.rate {
            current_config.rate = Rate::parse(&rate)?;
        }

        if let Some(source_denom) = config.source_denom {
            current_config.source_denom = Denom::new(source_denom)?;
        }

        if let Some(target_denom) = config.target_denom {
            current_config.target_denom = Denom::new(target_denom)?;
        }

        if let Some(paused) = config.paused {
            current_config.paused = paused;
        }

        // Ensure source and target denoms are not the same
        if current_config.source_denom == current_config.target_denom {
            return Err(ContractError::ConfigError(SameDenom));
        }

        CONFIG.save(deps.storage, &current_config)?;

        Ok(Response::new()
            .add_attribute("action", "update_config")
            .add_attribute("contract", CONTRACT_NAME)
            .add_attribute("version", CONTRACT_VERSION)
            .add_attribute("poa_admin", current_config.poa_admin)
            .add_attribute("rate", current_config.rate.to_string())
            .add_attribute("source_denom", current_config.source_denom.to_string())
            .add_attribute("target_denom", current_config.target_denom.to_string())
            .add_attribute("paused", current_config.paused.to_string()))
    }

    // Convert source tokens to target tokens
    // Steps:
    // 1. Validate that the sent funds are of the correct source_denom
    // 2. Send the source tokens to the POA admin address to be burned
    // 3. Calculate the amount of target tokens to mint based on the contract's rate
    // 4. Burn and mint tokens via AuthZ messages
    pub fn convert(deps: Deps, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        // Ensure contract is not paused
        if config.paused {
            return Err(ContractError::Paused);
        }

        // Funds (info.funds) are processed by the Bank module before reaching the contract
        // Ensure exactly one coin is sent
        let coin = one_coin(&info).map_err(|_| ContractError::ConvertError(InvalidFunds))?;

        // The coin should be of the source_denom type
        if coin.denom != config.source_denom.to_string() {
            return Err(ContractError::ConvertError(InvalidSourceDenom));
        }

        // Calculate amount to mint based on rate
        let amt_to_mint = config.rate.apply_to(coin.amount)?;

        // Send tokens to burn to the POA address
        let send = CosmosMsg::Bank(BankMsg::Send {
            to_address: config.poa_admin.to_string(),
            amount: vec![coin.clone()],
        });

        // Prepare to burn the tokens from the POA's held balance
        let burn = MsgBurnHeldBalance {
            authority: config.poa_admin.to_string(),
            burn_coins: vec![manifest_std::cosmos::base::v1beta1::Coin {
                denom: config.source_denom.to_string(),
                amount: coin.amount.to_string(),
            }],
        };
        let any_burn = Any {
            type_url: MsgBurnHeldBalance::TYPE_URL.to_string(),
            value: burn.encode_to_vec(),
        };

        // Prepare to mint new tokens to the sender's address
        let mint = MsgMint {
            sender: config.poa_admin.to_string(),
            amount: Some(manifest_std::cosmos::base::v1beta1::Coin {
                denom: config.target_denom.to_string(),
                amount: amt_to_mint.to_string(),
            }),
            mint_to_address: info.sender.to_string(),
        };
        let any_mint = Any {
            type_url: MsgMint::TYPE_URL.to_string(),
            value: mint.encode_to_vec(),
        };

        // Execute both burn and mint via AuthZ
        let exec = MsgExec {
            grantee: env.contract.address.to_string(),
            msgs: vec![any_burn, any_mint],
        };

        let msg = CosmosMsg::Any(AnyMsg {
            type_url: MsgExec::TYPE_URL.to_string(),
            value: exec.encode_to_vec().into(),
        });

        Ok(Response::new()
            .add_message(send)
            .add_message(msg)
            .add_attribute("action", "convert")
            .add_attribute("contract", CONTRACT_NAME)
            .add_attribute("version", CONTRACT_VERSION)
            .add_attribute("sender", info.sender)
            .add_attribute("poa_admin", config.poa_admin)
            .add_attribute("burned", coin.amount.to_string())
            .add_attribute("minted", amt_to_mint.to_string())
            .add_attribute("burned_denom", config.source_denom)
            .add_attribute("minted_denom", config.target_denom)
            .add_attribute("authz_grantee", env.contract.address)
            .add_attribute("authz_msg_count", "2")
            .add_attribute("burn_type", MsgBurnHeldBalance::TYPE_URL)
            .add_attribute("mint_type", MsgMint::TYPE_URL))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::consts::{BECH32_PREFIX, DEFAULT_POA_ADMIN};
//     use crate::denom::Denom;
//     use crate::msg::UpdateConfig;
//     use crate::rate::Rate;
//     use crate::state::Config;
//     use cosmwasm_std::testing::MockApi;
//     use cosmwasm_std::{Addr, Coin, CustomMsg, Empty};
//     use cw_controllers::AdminResponse;
//     use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor, StargateAccepting};
//
//     pub fn contract() -> Box<dyn Contract<Empty>> {
//         Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_migrate(migrate))
//     }
//
//     fn setup_default_app() -> (App, u64) {
//         let mut app = AppBuilder::default()
//             .with_api(MockApi::default().with_prefix(BECH32_PREFIX))
//             .build(|_, _, _| {});
//         let code_id = app.store_code(contract());
//         (app, code_id)
//     }
//
//     fn setup_app_with_funds(sender: &Addr, coins: Vec<Coin>) -> (App, u64) {
//         let mut app = AppBuilder::default()
//             .with_api(MockApi::default().with_prefix(BECH32_PREFIX))
//             .build(|router, _, storage| {
//                 router.bank.init_balance(storage, sender, coins).unwrap();
//             });
//         let code_id = app.store_code(contract());
//         (app, code_id)
//     }
//
//     fn instantiate_contract<T: CustomMsg + 'static, A: Executor<T>>(
//         app: &mut A,
//         code_id: u64,
//         admin: Addr,
//         config: Config,
//     ) -> StdResult<Addr> {
//         app.instantiate_contract(
//             code_id,
//             Addr::unchecked("creator"),
//             &InstantiateMsg {
//                 admin: admin.to_string(),
//                 poa_admin: config.poa_admin.to_string(),
//                 rate: config.rate.to_string(),
//                 source_denom: config.source_denom.to_string(),
//                 target_denom: config.target_denom.to_string(),
//                 paused: config.paused,
//             },
//             &[],
//             "test",
//             None,
//         )
//     }
//
//     #[test]
//     fn init() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin, config.clone())?;
//
//         let resp: Config = app.wrap().query_wasm_smart(&addr, &QueryMsg::Config {})?;
//         assert_eq!(resp.rate, config.rate);
//         assert_eq!(resp.poa_admin.as_str(), DEFAULT_POA_ADMIN);
//
//         Ok(())
//     }
//
//     #[test]
//     fn init_invalid_admin() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = Addr::unchecked("invalid");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let err = instantiate_contract(&mut app, code_id, admin, config).unwrap_err();
//         assert!(err.to_string().contains("parse failed"));
//         Ok(())
//     }
//
//     #[test]
//     fn init_invalid_rate() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse_unchecked("0")?)?;
//         let err = instantiate_contract(&mut app, code_id, admin, config).unwrap_err();
//         assert!(err.to_string().contains("invalid rate"));
//         assert!(err.to_string().contains("rate is zero"));
//         Ok(())
//     }
//
//     #[test]
//     fn init_same_denom() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let mut config = Config::try_with_defaults(Rate::parse("42")?)?;
//         config.target_denom = config.source_denom.clone();
//         let err = instantiate_contract(&mut app, code_id, admin, config).unwrap_err();
//         assert!(err
//             .to_string()
//             .contains("source and target denom cannot be the same"));
//         Ok(())
//     }
//
//     #[test]
//     fn update_config() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config.clone())?;
//
//         let new_poa_admin = app.api().addr_make("new_poa_admin");
//         let new_rate = "100".to_string();
//         let new_source = "umfx".to_string();
//         let new_target = "uatom".to_string();
//         let update_msg = ExecuteMsg::UpdateConfig {
//             config: UpdateConfig {
//                 poa_admin: Some(new_poa_admin.to_string()),
//                 rate: Some(new_rate.clone()),
//                 source_denom: Some(new_source.clone()),
//                 target_denom: Some(new_target.clone()),
//                 paused: Some(false),
//             },
//         };
//         let res = app.execute_contract(admin, addr.clone(), &update_msg, &[]);
//         assert!(res.is_ok());
//
//         let resp: Config = app.wrap().query_wasm_smart(&addr, &QueryMsg::Config {})?;
//         assert_eq!(resp.rate, Rate::parse(&new_rate)?);
//         assert_eq!(resp.poa_admin.to_string(), new_poa_admin.to_string());
//         assert_eq!(resp.source_denom, Denom::unchecked(new_source));
//         assert_eq!(resp.target_denom, Denom::unchecked(new_target));
//         Ok(())
//     }
//
//     #[test]
//     fn update_config_unauthorized() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin, config)?;
//
//         let unauthorized = app.api().addr_make("unauthorized");
//         let update_msg = ExecuteMsg::UpdateConfig {
//             config: UpdateConfig {
//                 paused: Some(false),
//                 ..Default::default()
//             },
//         };
//         let res = app.execute_contract(unauthorized.clone(), addr.clone(), &update_msg, &[]);
//         assert!(res.is_err());
//         let err = res.unwrap_err();
//         assert!(err.to_string().contains("unauthorized"));
//         assert!(err
//             .to_string()
//             .contains("only admin can perform this action"));
//         Ok(())
//     }
//
//     #[test]
//     fn update_config_invalid_rate() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let admin = app.api().addr_make("admin");
//
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config)?;
//
//         let update_msg = ExecuteMsg::UpdateConfig {
//             config: UpdateConfig {
//                 rate: Some("0".to_string()),
//                 ..Default::default()
//             },
//         };
//         let err = app
//             .execute_contract(admin, addr.clone(), &update_msg, &[])
//             .unwrap_err();
//         assert!(err.to_string().contains("invalid rate"));
//         assert!(err.to_string().contains("rate is zero"));
//         Ok(())
//     }
//
//     #[test]
//     fn update_config_empty_source_denom() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config.clone())?;
//
//         let update_msg = ExecuteMsg::UpdateConfig {
//             config: UpdateConfig {
//                 source_denom: Some("".to_string()),
//                 ..Default::default()
//             },
//         };
//         let err = app
//             .execute_contract(admin, addr, &update_msg, &[])
//             .unwrap_err();
//         assert!(err.to_string().contains("invalid denom"));
//         assert!(err.to_string().contains("denom is empty"));
//         Ok(())
//     }
//
//     #[test]
//     fn update_config_empty_target_denom() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config.clone())?;
//
//         let update_msg = ExecuteMsg::UpdateConfig {
//             config: UpdateConfig {
//                 target_denom: Some("".to_string()),
//                 ..Default::default()
//             },
//         };
//         let err = app
//             .execute_contract(admin, addr, &update_msg, &[])
//             .unwrap_err();
//         assert!(err.to_string().contains("invalid denom"));
//         assert!(err.to_string().contains("denom is empty"));
//         Ok(())
//     }
//
//     #[test]
//     fn update_config_partial() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config.clone())?;
//
//         let new_rate = "100".to_string();
//         let update_msg = ExecuteMsg::UpdateConfig {
//             config: UpdateConfig {
//                 rate: Some(new_rate.clone()),
//                 ..Default::default()
//             },
//         };
//         let res = app.execute_contract(admin.clone(), addr.clone(), &update_msg, &[]);
//         assert!(res.is_ok());
//
//         let resp: Config = app.wrap().query_wasm_smart(&addr, &QueryMsg::Config {})?;
//         assert_eq!(resp.rate, Rate::parse(&new_rate)?);
//         assert_eq!(resp.poa_admin.to_string(), config.poa_admin.to_string());
//         Ok(())
//     }
//
//     #[test]
//     fn no_update_config() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config.clone())?;
//
//         let update_msg = ExecuteMsg::UpdateConfig {
//             config: UpdateConfig {
//                 ..Default::default()
//             },
//         };
//         let res = app.execute_contract(admin, addr.clone(), &update_msg, &[]);
//         assert!(res.is_ok());
//
//         let resp: Config = app.wrap().query_wasm_smart(&addr, &QueryMsg::Config {})?;
//         assert_eq!(resp.rate, config.rate);
//         assert_eq!(resp.poa_admin.to_string(), config.poa_admin.to_string());
//
//         let app_response = res?;
//         let ev = app_response
//             .events
//             .iter()
//             .find(|ev| ev.ty == "wasm")
//             .unwrap();
//         assert_eq!(
//             ev.attributes
//                 .iter()
//                 .find(|attr| attr.key == "note")
//                 .unwrap()
//                 .value,
//             "empty config, no changes made"
//         );
//         Ok(())
//     }
//
//     #[test]
//     fn noop_update_config() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config.clone())?;
//
//         let update_msg = ExecuteMsg::UpdateConfig {
//             config: UpdateConfig {
//                 poa_admin: Some(config.poa_admin.to_string()),
//                 rate: Some(config.rate.to_string()),
//                 source_denom: Some(config.source_denom.to_string()),
//                 target_denom: Some(config.target_denom.to_string()),
//                 paused: Some(config.paused),
//             },
//         };
//         let res = app.execute_contract(admin, addr.clone(), &update_msg, &[]);
//         assert!(res.is_ok());
//
//         let resp: Config = app.wrap().query_wasm_smart(&addr, &QueryMsg::Config {})?;
//         assert_eq!(resp.rate, config.rate);
//         assert_eq!(resp.poa_admin.to_string(), config.poa_admin.to_string());
//         assert_eq!(resp.source_denom, config.source_denom);
//         assert_eq!(resp.target_denom, config.target_denom);
//         assert_eq!(resp.paused, config.paused);
//
//         let app_response = res?;
//         let ev = app_response
//             .events
//             .iter()
//             .find(|ev| ev.ty == "wasm")
//             .unwrap();
//         assert_eq!(
//             ev.attributes
//                 .iter()
//                 .find(|attr| attr.key == "note")
//                 .unwrap()
//                 .value,
//             "identical config, no changes made"
//         );
//         Ok(())
//     }
//
//     #[test]
//     fn update_config_reject_funds() -> Result<(), ContractError> {
//         let admin = MockApi::default()
//             .with_prefix(BECH32_PREFIX)
//             .addr_make("admin");
//         let coin = Coin {
//             denom: "umfx".to_string(),
//             amount: cosmwasm_std::Uint256::new(100),
//         };
//
//         let (mut app, code_id) = setup_app_with_funds(&admin, vec![coin]);
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config.clone())?;
//
//         let update_msg = ExecuteMsg::UpdateConfig {
//             config: UpdateConfig {
//                 paused: Some(false),
//                 ..Default::default()
//             },
//         };
//         let funds = vec![Coin {
//             denom: "umfx".to_string(),
//             amount: cosmwasm_std::Uint256::new(100),
//         }];
//         let err = app
//             .execute_contract(admin, addr.clone(), &update_msg, &funds)
//             .unwrap_err();
//         assert!(err.to_string().contains("non-payable"));
//         Ok(())
//     }
//
//     #[test]
//     fn convert_no_fund() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin, config.clone())?;
//
//         let sender = app.api().addr_make("sender");
//         let convert_msg = ExecuteMsg::Convert {};
//         let err = app
//             .execute_contract(sender, addr.clone(), &convert_msg, &[])
//             .unwrap_err();
//         assert!(err.to_string().contains("invalid funds sent"));
//         Ok(())
//     }
//
//     #[test]
//     fn convert_amount_is_zero() -> Result<(), ContractError> {
//         let sender = MockApi::default()
//             .with_prefix(BECH32_PREFIX)
//             .addr_make("sender");
//         let coin = Coin {
//             denom: "umfx".to_string(),
//             amount: cosmwasm_std::Uint256::new(100),
//         };
//
//         let (mut app, code_id) = setup_app_with_funds(&sender, vec![coin]);
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin, config.clone())?;
//
//         let convert_msg = ExecuteMsg::Convert {};
//         let funds = vec![Coin {
//             denom: "umfx".to_string(),
//             amount: cosmwasm_std::Uint256::new(0),
//         }];
//         let err = app
//             .execute_contract(sender, addr.clone(), &convert_msg, &funds)
//             .unwrap_err();
//         assert!(err
//             .to_string()
//             .contains("Cannot transfer empty coins amount"));
//         Ok(())
//     }
//
//     #[test]
//     fn convert_invalid_source_denom() -> Result<(), ContractError> {
//         let sender = Addr::unchecked("sender");
//         let coin = Coin {
//             denom: "invalid".to_string(),
//             amount: cosmwasm_std::Uint256::new(100),
//         };
//
//         let (mut app, code_id) = setup_app_with_funds(&sender, vec![coin]);
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin, config.clone())?;
//
//         let convert_msg = ExecuteMsg::Convert {};
//         let funds = vec![Coin {
//             denom: "invalid".to_string(),
//             amount: cosmwasm_std::Uint256::new(100),
//         }];
//         let err = app
//             .execute_contract(sender, addr.clone(), &convert_msg, &funds)
//             .unwrap_err();
//         assert!(err.to_string().contains("invalid source denom"));
//         Ok(())
//     }
//
//     #[test]
//     fn convert_invalid_multi_coin() -> Result<(), ContractError> {
//         let sender = Addr::unchecked("sender");
//         let coins = vec![
//             Coin {
//                 denom: "umfx".to_string(),
//                 amount: cosmwasm_std::Uint256::new(100),
//             },
//             Coin {
//                 denom: "uatom".to_string(),
//                 amount: cosmwasm_std::Uint256::new(100),
//             },
//         ];
//
//         let (mut app, code_id) = setup_app_with_funds(&sender, coins);
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin, config.clone())?;
//
//         let convert_msg = ExecuteMsg::Convert {};
//         let funds = vec![
//             Coin {
//                 denom: "umfx".to_string(),
//                 amount: cosmwasm_std::Uint256::new(50),
//             },
//             Coin {
//                 denom: "uatom".to_string(),
//                 amount: cosmwasm_std::Uint256::new(50),
//             },
//         ];
//         let err = app
//             .execute_contract(sender, addr.clone(), &convert_msg, &funds)
//             .unwrap_err();
//         assert!(err.to_string().contains("invalid funds sent"));
//         Ok(())
//     }
//
//     #[test]
//     fn paused() -> Result<(), ContractError> {
//         let sender = Addr::unchecked("sender");
//         let coin = Coin {
//             denom: "umfx".to_string(),
//             amount: cosmwasm_std::Uint256::new(100),
//         };
//
//         let (mut app, code_id) = setup_app_with_funds(&sender, vec![coin]);
//         let admin = app.api().addr_make("admin");
//         let mut config = Config::try_with_defaults(Rate::parse("42")?)?;
//         config.paused = true;
//         let addr = instantiate_contract(&mut app, code_id, admin, config.clone())?;
//
//         let convert_msg = ExecuteMsg::Convert {};
//         let funds = vec![Coin {
//             denom: "umfx".to_string(),
//             amount: cosmwasm_std::Uint256::new(100),
//         }];
//         let err = app
//             .execute_contract(sender, addr.clone(), &convert_msg, &funds)
//             .unwrap_err();
//         assert!(err.to_string().contains("contract is paused"));
//         Ok(())
//     }
//
//     #[test]
//     fn paused_unpaused() -> Result<(), ContractError> {
//         let sender = Addr::unchecked("sender");
//         let coin = Coin {
//             denom: "umfx".to_string(),
//             amount: cosmwasm_std::Uint256::new(100),
//         };
//
//         let mut app = AppBuilder::default()
//             .with_stargate(StargateAccepting) // Needed for AnyMsg, otherwise we get `Unexpected any execute: msg=AnyMsg`
//             .with_api(MockApi::default().with_prefix(BECH32_PREFIX))
//             .build(|router, _, storage| {
//                 router
//                     .bank
//                     .init_balance(storage, &sender, vec![coin.clone()])
//                     .unwrap();
//             });
//         let code_id = app.store_code(contract());
//         let admin = app.api().addr_make("admin");
//         let mut config = Config::try_with_defaults(Rate::parse("42")?)?;
//         config.paused = true;
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config.clone())?;
//
//         let convert_msg = ExecuteMsg::Convert {};
//         let funds = vec![Coin {
//             denom: "umfx".to_string(),
//             amount: cosmwasm_std::Uint256::new(100),
//         }];
//         let err = app
//             .execute_contract(sender.clone(), addr.clone(), &convert_msg, &funds)
//             .unwrap_err();
//         assert!(err.to_string().contains("contract is paused"));
//
//         // Unpause the contract
//         let update_msg = ExecuteMsg::UpdateConfig {
//             config: UpdateConfig {
//                 paused: Some(false),
//                 ..Default::default()
//             },
//         };
//         let res = app.execute_contract(admin, addr.clone(), &update_msg, &[]);
//         assert!(res.is_ok());
//
//         // Try converting again
//         let res = app.execute_contract(sender.clone(), addr.clone(), &convert_msg, &funds);
//         assert!(res.is_ok());
//         Ok(())
//     }
//
//     #[test]
//     fn update_admin() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config.clone())?;
//
//         let new_admin = app.api().addr_make("new_admin");
//         let update_msg = ExecuteMsg::UpdateAdmin {
//             admin: Some(new_admin.to_string()),
//         };
//         let res = app.execute_contract(admin.clone(), addr.clone(), &update_msg, &[]);
//         assert!(res.is_ok());
//
//         let resp: AdminResponse = app.wrap().query_wasm_smart(&addr, &QueryMsg::Admin {})?;
//         assert_eq!(resp.admin.unwrap(), new_admin.to_string());
//         Ok(())
//     }
//
//     #[test]
//     fn update_admin_unauthorized() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config.clone())?;
//
//         let unauthorized = app.api().addr_make("unauthorized");
//         let new_admin = app.api().addr_make("new_admin");
//         let update_msg = ExecuteMsg::UpdateAdmin {
//             admin: Some(new_admin.to_string()),
//         };
//         let res = app.execute_contract(unauthorized.clone(), addr.clone(), &update_msg, &[]);
//         assert!(res.is_err());
//         let err = res.unwrap_err();
//         assert!(err.to_string().contains("unauthorized"));
//         assert!(err
//             .to_string()
//             .contains("only admin can perform this action"));
//         Ok(())
//     }
//
//     #[test]
//     fn update_admin_reject_renounce() -> Result<(), ContractError> {
//         let (mut app, code_id) = setup_default_app();
//         let admin = app.api().addr_make("admin");
//         let config = Config::try_with_defaults(Rate::parse("42")?)?;
//         let addr = instantiate_contract(&mut app, code_id, admin.clone(), config.clone())?;
//
//         let update_msg = ExecuteMsg::UpdateAdmin { admin: None };
//         let res = app.execute_contract(admin, addr.clone(), &update_msg, &[]);
//         assert!(res.is_err());
//         let err = res.unwrap_err();
//         assert!(err.to_string().contains("cannot renounce admin role"));
//         Ok(())
//     }
// }
