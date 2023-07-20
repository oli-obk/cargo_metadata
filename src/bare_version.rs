//! This module defines the [`BareVersion`] type used for `rust_version` in [`Package`](crate::Package).
use std::{fmt::Display, num::ParseIntError, str::FromStr};

use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A bare version number with two or three components (used for `rust_version` in [`Package`](crate::Package)).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BareVersion {
    /// The major version component
    pub major: u64,
    /// The minor version component
    pub minor: u64,
    /// The patch version component
    pub patch: Option<u64>,
}

impl Display for BareVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.patch {
            Some(patch) => write!(f, "{}.{}.{}", self.major, self.minor, patch),
            None => write!(f, "{}.{}", self.major, self.minor),
        }
    }
}

impl FromStr for BareVersion {
    type Err = BareVersionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut parts = s.splitn(3, '.');
        Ok(Self {
            major: parts
                .next()
                .filter(|s| !s.is_empty())
                .ok_or(BareVersionError::Empty)?
                .parse()?,
            minor: parts
                .next()
                .filter(|s| !s.is_empty())
                .ok_or(BareVersionError::ExpectedMinorVersion)?
                .parse()?,
            patch: parts.next().map(u64::from_str).transpose()?,
        })
    }
}

impl Serialize for BareVersion {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for BareVersion {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let string = String::deserialize(d)?;
        string.parse().map_err(serde::de::Error::custom)
    }
}

/// An error while parsing a bare version.
#[derive(Debug)]
pub enum BareVersionError {
    /// Value is empty.
    Empty,
    /// Value lacks a minor version.
    ExpectedMinorVersion,
    /// Failed to parse an integer.
    ParseInt(ParseIntError),
}

impl Display for BareVersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BareVersionError::Empty => write!(f, "value is empty"),
            BareVersionError::ExpectedMinorVersion => write!(f, "expected a minor version"),
            BareVersionError::ParseInt(err) => write!(f, "failed to parse integer: {err}"),
        }
    }
}

impl From<ParseIntError> for BareVersionError {
    fn from(err: ParseIntError) -> Self {
        Self::ParseInt(err)
    }
}

// not used by this crate, just for the convenience of API users
impl From<Version> for BareVersion {
    fn from(value: Version) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
            patch: Some(value.patch),
        }
    }
}

// not used by this crate, just for the convenience of API users
impl From<BareVersion> for Version {
    fn from(value: BareVersion) -> Self {
        Self::new(value.major, value.minor, value.patch.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::bare_version::BareVersionError;

    use super::BareVersion;

    #[test]
    fn test_from_str() {
        assert!(matches!(
            BareVersion::from_str("1.2.3"),
            Ok(BareVersion {
                major: 1,
                minor: 2,
                patch: Some(3)
            })
        ));

        assert!(matches!(
            BareVersion::from_str("1.2"),
            Ok(BareVersion {
                major: 1,
                minor: 2,
                patch: None
            })
        ));

        assert!(matches!(
            BareVersion::from_str("1.2."),
            Err(BareVersionError::ParseInt(..))
        ));

        assert!(matches!(
            BareVersion::from_str("1."),
            Err(BareVersionError::ExpectedMinorVersion)
        ));

        assert!(matches!(
            BareVersion::from_str("1"),
            Err(BareVersionError::ExpectedMinorVersion)
        ));

        assert!(matches!(
            BareVersion::from_str(""),
            Err(BareVersionError::Empty)
        ));

        assert!(matches!(
            BareVersion::from_str(" "),
            Err(BareVersionError::ParseInt(..))
        ));
    }
}
