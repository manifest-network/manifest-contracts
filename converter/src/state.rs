use crate::consts::{default_source_denom, default_target_denom, DEFAULT_POA_ADMIN};
use crate::denom::Denom;
use crate::error::ConfigError::SameDenom;
use crate::error::ContractError;
use crate::rate::Rate;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;

// Never rename/remove fields from this struct, only add optional fields to avoid
// breaking changes. If you need to rename/remove a field, you must version the config
#[cw_serde]
pub struct Config {
    pub poa_admin: Addr,
    pub rate: Rate,
    pub source_denom: Denom,
    pub target_denom: Denom,
    pub paused: bool,
    // Future fields should be optional, e.g.
    //
    //   #[serde(default, skip_serializing_if = "Option::is_none")]
    //   pub min_amount: Option<Uint256>,
    //
    // If non-optional fields are added, config must be versioned and the migration handler must be updated
}

// Never rename the storage keys
pub const CONFIG: Item<Config> = Item::new("config");
pub const ADMIN: Admin = Admin::new("admin");

impl Config {
    pub fn try_with_defaults(rate: Rate) -> Result<Self, ContractError> {
        let s = default_source_denom();
        let t = default_target_denom();
        if s == t {
            return Err(ContractError::ConfigError(SameDenom));
        }
        Ok(Self {
            poa_admin: Addr::unchecked(DEFAULT_POA_ADMIN),
            rate,
            source_denom: s,
            target_denom: t,
            paused: false,
        })
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.source_denom == self.target_denom {
            return Err(ContractError::ConfigError(SameDenom));
        }
        Ok(())
    }
}
