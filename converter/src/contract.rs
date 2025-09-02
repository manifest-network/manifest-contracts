use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::CONFIG;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api.addr_validate(msg.config.admin.as_str())?;
    CONFIG.save(deps.storage, &msg.config)?;

    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Config {} => to_json_binary(&query::config(deps)?),
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
        UpdateConfig { config } => exec::update_config(deps, info, config),
        Convert {} => exec::convert(deps, env, info),
    }
}

mod helper {
    pub(crate) fn to_manifest_coin(
        c: &cosmwasm_std::Coin,
    ) -> manifest_std::cosmos::base::v1beta1::Coin {
        manifest_std::cosmos::base::v1beta1::Coin {
            denom: c.denom.to_string(),
            amount: c.amount.to_string(),
        }
    }
}

mod query {
    use super::*;
    use crate::msg::ConfigResp;

    pub fn config(deps: Deps) -> StdResult<ConfigResp> {
        let config = CONFIG.load(deps.storage)?;
        Ok(ConfigResp { config })
    }
}

mod exec {
    use super::*;
    use crate::denom::Denom;
    use crate::error::AuthError;
    use crate::error::ConfigError::SameDenom;
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

    // Update the contract configuration with new values
    pub fn update_config(
        deps: DepsMut,
        info: MessageInfo,
        config: UpdateConfig,
    ) -> Result<Response, ContractError> {
        if config.is_empty() {
            return Ok(Response::new()
                .add_attribute("action", "update_config")
                .add_attribute("note", "no changes made"));
        }
        let mut current_config = CONFIG.load(deps.storage)?;

        if info.sender != current_config.admin {
            return Err(ContractError::Unauthorized(AuthError::NotAdmin));
        }

        if let Some(admin) = config.admin {
            let admin_addr = deps.api.addr_validate(admin.as_str())?;
            current_config.admin = admin_addr;
        }

        if let Some(poa_admin) = config.poa_admin {
            let poa_admin_addr = deps.api.addr_validate(poa_admin.as_str())?;
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

        // Ensure source and target denoms are not the same
        if current_config.source_denom == current_config.target_denom {
            return Err(ContractError::ConfigError(SameDenom));
        }

        CONFIG.save(deps.storage, &current_config)?;

        Ok(Response::new()
            .add_attribute("action", "update_config")
            .add_attribute("admin", current_config.admin.into_string())
            .add_attribute("poa_admin", current_config.poa_admin.into_string())
            .add_attribute("rate", current_config.rate.to_string())
            .add_attribute("source_denom", current_config.source_denom)
            .add_attribute("target_denom", current_config.target_denom))
    }

    // Convert source tokens to target tokens
    // Steps:
    // 1. Validate that the sent funds are of the correct source_denom
    // 2. Send the source tokens to the POA admin address to be burned
    // 3. Calculate the amount of target tokens to mint based on the contract's rate
    // 4. Burn and mint tokens via AuthZ messages
    pub fn convert(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

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
            burn_coins: vec![helper::to_manifest_coin(&coin)],
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
            value: Binary::from(exec.encode_to_vec()),
        });

        Ok(Response::new()
            .add_message(send)
            .add_message(msg)
            .add_attribute("action", "convert")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("poa_admin", config.poa_admin.into_string())
            .add_attribute("burned", coin.amount.to_string())
            .add_attribute("minted", amt_to_mint.to_string())
            .add_attribute("burned_denom", config.source_denom)
            .add_attribute("minted_denom", config.target_denom))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::denom::Denom;
    use crate::msg::{ConfigResp, UpdateConfig};
    use crate::rate::Rate;
    use crate::state::Config;
    use cosmwasm_std::{Addr, CustomMsg, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    pub fn contract() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query))
    }

    fn setup_app() -> (App, u64) {
        let mut app = App::default();
        let code_id = app.store_code(contract());
        (app, code_id)
    }

    fn instantiate_contract<T: CustomMsg + 'static, A: Executor<T>>(
        app: &mut A,
        code_id: u64,
        config: Config,
    ) -> StdResult<Addr> {
        app.instantiate_contract(
            code_id,
            Addr::unchecked("creator"),
            &InstantiateMsg { config },
            &[],
            "test",
            None,
        )
    }

    #[test]
    fn init() {
        let (mut app, code_id) = setup_app();
        let config =
            Config::with_defaults(app.api().addr_make("admin"), Rate::parse("42").unwrap());
        let addr = instantiate_contract(&mut app, code_id, config.clone()).unwrap();

        let resp: ConfigResp = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::Config {})
            .unwrap();
        assert_eq!(resp.config.rate, config.rate);
        assert_eq!(resp.config.admin.to_string(), config.admin.to_string());
    }

    #[test]
    fn init_invalid_admin() {
        let (mut app, code_id) = setup_app();
        let config = Config::with_defaults(Addr::unchecked("invalid"), Rate::parse("42").unwrap());
        let err = instantiate_contract(&mut app, code_id, config).unwrap_err();
        assert!(err.to_string().contains("parse failed"));
    }

    #[test]
    fn init_invalid_rate() {
        let (mut app, code_id) = setup_app();
        let config = Config::with_defaults(
            app.api().addr_make("admin"),
            Rate::parse_unchecked("0").unwrap(),
        );
        let err = instantiate_contract(&mut app, code_id, config).unwrap_err();
        assert!(err.to_string().contains("invalid rate"));
        assert!(err.to_string().contains("rate is zero"));
    }

    #[test]
    fn update_config() {
        let (mut app, code_id) = setup_app();
        let config =
            Config::with_defaults(app.api().addr_make("admin"), Rate::parse("42").unwrap());
        let addr = instantiate_contract(&mut app, code_id, config.clone()).unwrap();

        let new_admin = app.api().addr_make("new_admin");
        let new_poa_admin = app.api().addr_make("new_poa_admin");
        let new_rate = "100".to_string();
        let new_source = "umfx".to_string();
        let new_target = "uatom".to_string();
        let update_msg = ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                admin: Some(new_admin.to_string()),
                poa_admin: Some(new_poa_admin.to_string()),
                rate: Some(new_rate.clone()),
                source_denom: Some(new_source.clone()),
                target_denom: Some(new_target.clone()),
            },
        };
        let res = app.execute_contract(config.admin, addr.clone(), &update_msg, &[]);
        assert!(res.is_ok());

        let resp: ConfigResp = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::Config {})
            .unwrap();
        assert_eq!(resp.config.rate, Rate::parse(&new_rate).unwrap());
        assert_eq!(resp.config.admin.to_string(), new_admin.to_string());
        assert_eq!(resp.config.poa_admin.to_string(), new_poa_admin.to_string());
        assert_eq!(resp.config.source_denom, Denom::unchecked(new_source));
        assert_eq!(resp.config.target_denom, Denom::unchecked(new_target));
    }

    #[test]
    fn update_config_unauthorized() {
        let (mut app, code_id) = setup_app();
        let config =
            Config::with_defaults(app.api().addr_make("admin"), Rate::parse("42").unwrap());
        let addr = instantiate_contract(&mut app, code_id, config).unwrap();

        let unauthorized = app.api().addr_make("unauthorized");
        let new_admin = app.api().addr_make("new_admin");
        let update_msg = ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                admin: Some(new_admin.to_string()),
                ..Default::default()
            },
        };
        let res = app.execute_contract(unauthorized.clone(), addr.clone(), &update_msg, &[]);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.to_string().contains("unauthorized"));
        assert!(err
            .to_string()
            .contains("only admin can perform this action"));
    }

    #[test]
    fn update_config_invalid_rate() {
        let (mut app, code_id) = setup_app();
        let config =
            Config::with_defaults(app.api().addr_make("admin"), Rate::parse("42").unwrap());
        let admin = app.api().addr_make("admin");

        let addr = instantiate_contract(&mut app, code_id, config).unwrap();

        let update_msg = ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                rate: Some("0".to_string()),
                ..Default::default()
            },
        };
        let err = app
            .execute_contract(admin.clone(), addr.clone(), &update_msg, &[])
            .unwrap_err();
        assert!(err.to_string().contains("invalid rate"));
        assert!(err.to_string().contains("rate is zero"));
    }

    #[test]
    fn update_config_invalid_admin() {
        let (mut app, code_id) = setup_app();
        let config =
            Config::with_defaults(app.api().addr_make("admin"), Rate::parse("42").unwrap());
        let addr = instantiate_contract(&mut app, code_id, config.clone()).unwrap();

        let update_msg = ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                admin: Some("invalid_admin".to_string()),
                ..Default::default()
            },
        };
        let err = app
            .execute_contract(config.admin, addr.clone(), &update_msg, &[])
            .unwrap_err();
        assert!(err.to_string().contains("parse failed"));
    }

    #[test]
    fn update_config_empty_source_denom() {
        let (mut app, code_id) = setup_app();
        let config =
            Config::with_defaults(app.api().addr_make("admin"), Rate::parse("42").unwrap());
        let addr = instantiate_contract(&mut app, code_id, config.clone()).unwrap();

        let update_msg = ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                source_denom: Some("".to_string()),
                ..Default::default()
            },
        };
        let err = app
            .execute_contract(config.admin.clone(), addr.clone(), &update_msg, &[])
            .unwrap_err();
        assert!(err.to_string().contains("invalid denom"));
        assert!(err.to_string().contains("denom is empty"));
    }

    #[test]
    fn update_config_empty_target_denom() {
        let (mut app, code_id) = setup_app();
        let config =
            Config::with_defaults(app.api().addr_make("admin"), Rate::parse("42").unwrap());
        let addr = instantiate_contract(&mut app, code_id, config.clone()).unwrap();

        let update_msg = ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                target_denom: Some("".to_string()),
                ..Default::default()
            },
        };
        let err = app
            .execute_contract(config.admin.clone(), addr.clone(), &update_msg, &[])
            .unwrap_err();
        assert!(err.to_string().contains("invalid denom"));
        assert!(err.to_string().contains("denom is empty"));
    }

    #[test]
    fn update_config_partial() {
        let (mut app, code_id) = setup_app();
        let config =
            Config::with_defaults(app.api().addr_make("admin"), Rate::parse("42").unwrap());
        let addr = instantiate_contract(&mut app, code_id, config.clone()).unwrap();

        let new_rate = "100".to_string();
        let update_msg = ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                rate: Some(new_rate.clone()),
                ..Default::default()
            },
        };
        let res = app.execute_contract(config.admin.clone(), addr.clone(), &update_msg, &[]);
        assert!(res.is_ok());

        let resp: ConfigResp = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::Config {})
            .unwrap();
        assert_eq!(resp.config.rate, Rate::parse(&new_rate).unwrap());
        assert_eq!(resp.config.admin.to_string(), config.admin.to_string());
    }

    #[test]
    fn no_update_config() {
        let (mut app, code_id) = setup_app();
        let config =
            Config::with_defaults(app.api().addr_make("admin"), Rate::parse("42").unwrap());
        let addr = instantiate_contract(&mut app, code_id, config.clone()).unwrap();

        let update_msg = ExecuteMsg::UpdateConfig {
            config: UpdateConfig {
                ..Default::default()
            },
        };
        let res = app.execute_contract(config.admin.clone(), addr.clone(), &update_msg, &[]);
        assert!(res.is_ok());

        let resp: ConfigResp = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::Config {})
            .unwrap();
        assert_eq!(resp.config.rate, config.rate);
        assert_eq!(resp.config.admin.to_string(), config.admin.to_string());
    }

    #[test]
    fn convert_no_fund() {
        let (mut app, code_id) = setup_app();
        let config =
            Config::with_defaults(app.api().addr_make("admin"), Rate::parse("42").unwrap());
        let addr = instantiate_contract(&mut app, code_id, config.clone()).unwrap();

        let sender = app.api().addr_make("sender");
        let convert_msg = ExecuteMsg::Convert {};
        let err = app
            .execute_contract(sender, addr.clone(), &convert_msg, &[])
            .unwrap_err();
        assert!(err.to_string().contains("invalid funds sent"));
    }

    #[test]
    fn convert_invalid_source_denom() {
        let sender = Addr::unchecked("sender");
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &sender,
                    vec![cosmwasm_std::Coin {
                        denom: "invalid".to_string(),
                        amount: cosmwasm_std::Uint256::new(100),
                    }],
                )
                .unwrap();
        });
        let code_id = app.store_code(contract());

        let config =
            Config::with_defaults(app.api().addr_make("admin"), Rate::parse("42").unwrap());
        let addr = instantiate_contract(&mut app, code_id, config.clone()).unwrap();

        let convert_msg = ExecuteMsg::Convert {};
        let funds = vec![cosmwasm_std::Coin {
            denom: "invalid".to_string(),
            amount: cosmwasm_std::Uint256::new(100),
        }];
        let err = app
            .execute_contract(sender, addr.clone(), &convert_msg, &funds)
            .unwrap_err();
        assert!(err.to_string().contains("invalid source denom"));
    }
}
