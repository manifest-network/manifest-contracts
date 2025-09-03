use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    StdError(#[from] StdError),
    #[error("unauthorized: {0}")]
    Unauthorized(#[from] AuthError),
    #[error("invalid rate: {0}")]
    RateError(#[from] RateError),
    #[error("invalid denom: {0}")]
    DenomError(#[from] DenomError),
    #[error("invalid amount: {0}")]
    AmountError(#[from] AmountError),
    #[error("conversion error: {0}")]
    ConvertError(#[from] ConvertError),
    #[error("configuration error: {0}")]
    ConfigError(#[from] ConfigError),
    #[error("migration error: {0}")]
    MigrateError(#[from] MigrateError),
    #[error("contract is paused")]
    Paused,
}

#[derive(Error, Debug)]
pub enum RateError {
    #[error("rate is zero")]
    InvalidRateZero,
    #[error("failed to parse rate")]
    InvalidRateParsing,
    #[error("failed to apply rate")]
    ApplyOverflowError,
    #[error("resulting amount is zero")]
    ApplyZeroError,
}

#[derive(Error, Debug)]
pub enum AmountError {
    #[error("amount is zero")]
    AmountIsZero,
    #[error("amount exceeds maximum")]
    AmountExceedsMax,
    #[error("failed to parse amount")]
    InvalidAmountParsing,
    #[error("non-payable function called with funds")]
    NonPayable,
}

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("invalid funds sent")]
    InvalidFunds,
    #[error("invalid source denom")]
    InvalidSourceDenom,
}

#[derive(Error, Debug)]
pub enum DenomError {
    #[error("denom is empty")]
    EmptyDenom,
    #[error("invalid ibc denom format")]
    InvalidIbcDenomFormat,
    #[error("invalid factory denom format")]
    InvalidFactoryDenomFormat,
    #[error("invalid denom format")]
    InvalidDenomFormat,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("only admin can perform this action")]
    NotAdmin,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("source and target denom cannot be the same")]
    SameDenom,
}

#[derive(Error, Debug)]
pub enum MigrateError {
    #[error("invalid contract name")]
    InvalidContractName,
}
