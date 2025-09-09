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
