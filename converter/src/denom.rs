use crate::error::{ContractError, DenomError};
use cosmwasm_schema::cw_serde;

type DenomInner = String;

#[cw_serde]
#[schemars(with = "DenomInner")]
#[serde(try_from = "DenomInner", into = "DenomInner")]
#[schemaifier(mute_warnings)]
pub struct Denom(DenomInner);

impl Denom {
    #[inline]
    pub fn validate(&self) -> Result<(), ContractError> {
        let s = self.as_str();
        if s.is_empty() {
            return Err(ContractError::DenomError(DenomError::EmptyDenom));
        }

        if s.starts_with("ibc/") {
            if !is_ibc(s) {
                return Err(ContractError::DenomError(DenomError::InvalidIbcDenomFormat));
            }
        } else if s.starts_with("factory/") {
            if !is_factory(s) {
                return Err(ContractError::DenomError(
                    DenomError::InvalidFactoryDenomFormat,
                ));
            }
        } else if !is_native(s) {
            return Err(ContractError::DenomError(DenomError::InvalidDenomFormat));
        }
        Ok(())
    }

    #[inline]
    pub fn new(denom: impl Into<DenomInner>) -> Result<Self, ContractError> {
        let d = Denom(denom.into());
        d.validate()?;
        Ok(d)
    }

    #[inline]
    pub fn unchecked(denom: impl Into<DenomInner>) -> Self {
        Denom(denom.into())
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[inline]
    pub fn into_inner(self) -> DenomInner {
        self.0
    }
}

#[inline]
fn is_native(s: &str) -> bool {
    if !(s.len() >= 3 && s.len() <= 128) {
        return false;
    }
    let mut chars = s.chars();
    if chars.next() != Some('u') {
        return false;
    }
    chars.all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
}

#[inline]
fn is_ibc(s: &str) -> bool {
    const PREF: &str = "ibc/";
    let hash = &s[PREF.len()..];
    if hash.len() != 64 {
        return false;
    }
    hash.bytes()
        .all(|b| b.is_ascii_digit() || (b'A'..=b'F').contains(&b))
}

#[inline]
fn is_factory(s: &str) -> bool {
    const PREF: &str = "factory/";
    let rest = &s[PREF.len()..];
    let mut it = rest.split('/');
    let Some(creator) = it.next() else {
        return false;
    };
    let Some(sub) = it.next() else {
        return false;
    };
    if it.next().is_some() {
        return false;
    }
    if bech32::decode(creator).is_err() {
        return false;
    }

    // subdenom chars allowed by SDK: [A-Za-z0-9/.:_-]; here we disallow '/' because we already split:
    if sub.is_empty() || sub.len() > 128 {
        return false;
    }
    sub.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | ':' | '_' | '-'))
}

impl From<Denom> for DenomInner {
    fn from(value: Denom) -> Self {
        value.0
    }
}

impl TryFrom<DenomInner> for Denom {
    type Error = ContractError;
    fn try_from(value: DenomInner) -> Result<Self, Self::Error> {
        Denom::new(value)
    }
}

impl std::fmt::Display for Denom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Denom;
    use crate::error::{ContractError, DenomError};

    #[test]
    fn test_denom() {
        let d = Denom::new("uatom").unwrap();
        assert_eq!(d.into_inner(), "uatom");
    }

    #[test]
    fn test_denom_empty() {
        let err = Denom::new("").unwrap_err();
        assert!(matches!(
            err,
            ContractError::DenomError(DenomError::EmptyDenom)
        ));
    }

    #[test]
    fn test_denom_invalid() {
        let err = Denom::new("atom").unwrap_err();
        assert!(matches!(
            err,
            ContractError::DenomError(DenomError::InvalidDenomFormat)
        ));
    }

    #[test]
    fn test_denom_ibc() {
        let ibc = "ibc/E91A88D2F4A515E48A183869B10B7C20A73F6DEE1BBE864FD15924EADB8A078F";
        let d = Denom::new(ibc).unwrap();
        assert_eq!(d.into_inner(), ibc);
    }

    #[test]
    fn test_denom_ibc_invalid() {
        let err = Denom::new("ibc/invalidhash").unwrap_err();
        assert!(matches!(
            err,
            ContractError::DenomError(DenomError::InvalidIbcDenomFormat)
        ));
    }

    #[test]
    fn test_denom_factory() {
        let factory =
            "factory/manifest1afk9zr2hn2jsac63h4hm60vl9z3e5u69gndzf7c99cqge3vzwjzsfmy9qj/upwr";
        let d = Denom::new(factory).unwrap();
        assert_eq!(d.into_inner(), factory);
    }

    #[test]
    fn test_denom_factory_invalid() {
        let err = Denom::new("factory/invalid").unwrap_err();
        assert!(matches!(
            err,
            ContractError::DenomError(DenomError::InvalidFactoryDenomFormat)
        ));
    }

    #[test]
    fn test_denom_factory_invalid_bech32() {
        let err = Denom::new("factory/invalid_bech32/upwr").unwrap_err();
        assert!(matches!(
            err,
            ContractError::DenomError(DenomError::InvalidFactoryDenomFormat)
        ));
    }
}
