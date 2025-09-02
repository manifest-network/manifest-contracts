use crate::state::Config;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub config: Config,
}

#[cw_serde]
pub enum ExecuteMsg {
    Convert {},
    UpdateConfig { config: UpdateConfig },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
}

// TODO: Write a macro to generate this struct from the Config struct
#[cw_serde]
#[derive(Default)]
pub struct UpdateConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poa_admin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_denom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_denom: Option<String>,
}

impl UpdateConfig {
    pub fn is_empty(&self) -> bool {
        self.admin.is_none()
            && self.rate.is_none()
            && self.source_denom.is_none()
            && self.target_denom.is_none()
    }
}

#[cw_serde]
pub struct ConfigResp {
    pub config: Config,
}
