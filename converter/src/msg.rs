use crate::state::Config;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub poa_admin: String,
    pub rate: String,
    pub source_denom: String,
    pub target_denom: String,
    pub paused: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    Convert {},
    UpdateConfig { config: UpdateConfig },
    UpdateAdmin { admin: Option<String> },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Admin {},
}

#[cw_serde]
pub enum MigrateMsg {}

// TODO: Write a macro to generate this struct from the Config struct
#[cw_serde]
#[derive(Default)]
pub struct UpdateConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poa_admin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_denom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_denom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
}

impl UpdateConfig {
    // Check if no fields are set in this update
    pub fn is_empty(&self) -> bool {
        self.poa_admin.is_none()
            && self.rate.is_none()
            && self.source_denom.is_none()
            && self.target_denom.is_none()
            && self.paused.is_none()
    }

    // Check if applying this update to the given config would result in no changes
    pub fn is_noop(&self, other: &Config) -> bool {
        (self.poa_admin.is_none()
            || self
                .poa_admin
                .as_ref()
                .map(|a| a == other.poa_admin.as_str())
                .unwrap_or(true))
            && (self.rate.is_none()
                || self
                    .rate
                    .as_ref()
                    .map(|r| r == &other.rate.as_ref().to_string())
                    .unwrap_or(true))
            && (self.source_denom.is_none()
                || self
                    .source_denom
                    .as_ref()
                    .map(|d| d == other.source_denom.as_str())
                    .unwrap_or(true))
            && (self.target_denom.is_none()
                || self
                    .target_denom
                    .as_ref()
                    .map(|d| d == other.target_denom.as_str())
                    .unwrap_or(true))
            && (self.paused.is_none() || self.paused.map(|p| p == other.paused).unwrap_or(true))
    }
}
