use crate::denom::Denom;
use const_format::formatcp;

pub const CONTRACT_NAME: &str = "manifest/converter";

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const BECH32_PREFIX: &str = "manifest";

// The default POA admin address of the Manifest Network
pub const DEFAULT_POA_ADMIN: &str =
    formatcp!("{BECH32_PREFIX}1afk9zr2hn2jsac63h4hm60vl9z3e5u69gndzf7c99cqge3vzwjzsfmy9qj");

// The default base denom of the Manifest Network
pub const DEFAULT_SOURCE_DENOM: &str = "umfx";

// The default target denom of the Manifest Network
pub const DEFAULT_TARGET_DENOM: &str = formatcp!("factory/{DEFAULT_POA_ADMIN}/upwr");

pub fn default_source_denom() -> Denom {
    Denom::unchecked(DEFAULT_SOURCE_DENOM)
}

pub fn default_target_denom() -> Denom {
    Denom::unchecked(DEFAULT_TARGET_DENOM)
}
