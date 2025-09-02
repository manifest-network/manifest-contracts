use crate::error::AmountError::{AmountExceedsMax, AmountIsZero};
use crate::error::ContractError;
use crate::error::RateError::{
    ApplyOverflowError, ApplyZeroError, InvalidRateParsing, InvalidRateZero,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal256, Uint256};
use std::str::FromStr;

type RateInner = Decimal256;

#[cw_serde]
#[schemars(with = "RateInner")]
#[schemaifier(mute_warnings)]
#[serde(try_from = "RateInner", into = "RateInner")]
pub struct Rate(RateInner);
impl Rate {
    #[inline]
    fn validate(&self) -> Result<(), ContractError> {
        if !self.0.is_zero() {
            Ok(())
        } else {
            Err(ContractError::RateError(InvalidRateZero))
        }
    }

    #[inline]
    pub fn new(rate: RateInner) -> Result<Self, ContractError> {
        let r = Rate(rate);
        r.validate()?;
        Ok(r)
    }

    #[inline]
    pub fn into_inner(self) -> RateInner {
        self.0
    }

    #[inline]
    pub fn as_ref(&self) -> &RateInner {
        &self.0
    }

    fn _parse(s: &str) -> Result<Decimal256, ContractError> {
        s.parse::<Decimal256>()
            .map_err(|_| ContractError::RateError(InvalidRateParsing))
    }

    #[inline]
    pub fn parse(s: &str) -> Result<Self, ContractError> {
        Self::new(Self::_parse(s)?)
    }

    #[inline]
    pub fn parse_unchecked(s: &str) -> Result<Self, ContractError> {
        Ok(Rate(Self::_parse(s)?))
    }

    #[inline]
    pub fn apply_to(&self, amount: impl Into<Uint256>) -> Result<Uint256, ContractError> {
        let amount = amount.into();
        if amount.is_zero() {
            return Err(ContractError::AmountError(AmountIsZero));
        }
        let amount_dec = Decimal256::from_atomics(amount, 0)
            .map_err(|_| ContractError::AmountError(AmountExceedsMax))?;
        let res = self
            .0
            .checked_mul(amount_dec)
            .map_err(|_| ContractError::RateError(ApplyOverflowError))?;

        let floor = res.to_uint_floor();
        if floor.is_zero() {
            return Err(ContractError::RateError(ApplyZeroError));
        }
        Ok(floor)
    }
}

impl From<Rate> for RateInner {
    fn from(value: Rate) -> Self {
        value.0
    }
}

impl std::fmt::Display for Rate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<RateInner> for Rate {
    type Error = ContractError;
    fn try_from(value: RateInner) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for Rate {
    type Err = ContractError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Rate::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::{Rate, RateInner};
    use crate::error::RateError::ApplyZeroError;
    use crate::error::{AmountError, ContractError, RateError};
    use cosmwasm_std::{Uint128, Uint256};
    use std::str::FromStr;

    #[test]
    fn test_rate() {
        let v = "100";
        assert_eq!(
            Rate::parse(v).unwrap().into_inner(),
            RateInner::from_str(v).unwrap()
        );
    }

    #[test]
    fn test_rate_decimal() {
        let v = "0.000001";
        assert_eq!(
            Rate::parse(v).unwrap().into_inner(),
            RateInner::from_str(v).unwrap()
        );
    }

    #[test]
    fn test_rate_max() {
        let v = RateInner::MAX.to_string();
        assert_eq!(
            Rate::parse(&v).unwrap().into_inner(),
            RateInner::from_str(&v).unwrap()
        );
    }

    #[test]
    fn test_rate_min() {
        assert!(matches!(
            Rate::new(RateInner::MIN).unwrap_err(),
            ContractError::RateError(RateError::InvalidRateZero)
        ));
    }

    #[test]
    fn test_rate_too_many_decimals() {
        assert!(matches!(
            Rate::parse("0.0000000000000000001").unwrap_err(),
            ContractError::RateError(RateError::InvalidRateParsing)
        ));
    }

    #[test]
    fn test_rate_invalid() {
        assert!(matches!(
            Rate::parse("0").unwrap_err(),
            ContractError::RateError(RateError::InvalidRateZero)
        ));
    }

    #[test]
    fn test_rate_invalid_parse() {
        assert!(matches!(
            Rate::parse("invalid").unwrap_err(),
            ContractError::RateError(RateError::InvalidRateParsing)
        ));
    }

    #[test]
    fn test_rate_negative() {
        assert!(matches!(
            Rate::parse("-1").unwrap_err(),
            ContractError::RateError(RateError::InvalidRateParsing)
        ));
    }

    #[test]
    fn test_rate_apply_to() {
        let r = Rate::parse("1.5").unwrap();
        assert_eq!(r.apply_to(100u8).unwrap(), Uint256::from(150u8));
        assert_eq!(
            r.apply_to(Uint128::new(100)).unwrap(),
            Uint256::from(150u128)
        );
        assert_eq!(
            r.apply_to(Uint256::from(100u128)).unwrap(),
            Uint256::from(150u128)
        );
    }

    #[test]
    fn test_rate_apply_to_zero_amount() {
        let r = Rate::parse("1.5").unwrap();
        assert!(matches!(
            r.apply_to(0u8).unwrap_err(),
            ContractError::AmountError(AmountError::AmountIsZero)
        ));
    }

    #[test]
    fn test_rate_apply_to_overflow() {
        let r = Rate::parse(&RateInner::MAX.to_string()).unwrap();
        assert!(matches!(
            r.apply_to(2u8).unwrap_err(),
            ContractError::RateError(RateError::ApplyOverflowError)
        ));

        let r = Rate::parse("1.000000000000000001").unwrap();
        assert!(matches!(
            r.apply_to(Uint256::MAX).unwrap_err(),
            ContractError::AmountError(AmountError::AmountExceedsMax)
        ));
    }

    #[test]
    fn test_rate_apply_to_work() {
        let r = Rate::parse("0.379").unwrap();
        assert_eq!(r.apply_to(1000000u32).unwrap(), Uint256::from(379000u32));
        assert_eq!(r.apply_to(1000u16).unwrap(), Uint256::from(379u16));
        assert_eq!(r.apply_to(100u8).unwrap(), Uint256::from(37u8));
        assert_eq!(r.apply_to(10u8).unwrap(), Uint256::from(3u8));
        assert!(matches!(
            r.apply_to(1u8).unwrap_err(),
            ContractError::RateError(ApplyZeroError)
        ));
    }

    #[test]
    fn test_rate_apply_to_zero_result() {
        assert!(matches!(
            Rate::parse("0.0001").unwrap().apply_to(1u8).unwrap_err(),
            ContractError::RateError(ApplyZeroError)
        ));
    }
}
