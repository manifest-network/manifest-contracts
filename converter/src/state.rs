use crate::consts::{default_source_denom, default_target_denom, DEFAULT_POA_ADMIN};
use crate::denom::Denom;
use crate::rate::Rate;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub poa_admin: Addr,
    pub rate: Rate,
    pub source_denom: Denom,
    pub target_denom: Denom,
}

pub const CONFIG: Item<Config> = Item::new("config");

impl Config {
    pub fn with_defaults(admin: Addr, rate: Rate) -> Self {
        assert_ne!(default_target_denom(), default_source_denom());
        Self {
            admin,
            poa_admin: Addr::unchecked(DEFAULT_POA_ADMIN),
            rate,
            source_denom: default_source_denom(),
            target_denom: default_target_denom(),
        }
    }
}
