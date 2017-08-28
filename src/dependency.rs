//! This module contains `Dependency` and the types/functions it uses for deserialization.

use serde::{Deserialize, Deserializer};
use semver::VersionReq;

#[derive(PartialEq, Clone, Debug, Copy, Deserialize)]
/// Dependencies can come in three kinds
pub enum DependencyKind {
    #[serde(rename = "normal")]
    /// The 'normal' kind
    Normal,
    #[serde(rename = "dev")]
    /// Those used in tests only
    Development,
    #[serde(rename = "build")]
    /// Those used in build scripts only
    Build,
}

impl Default for DependencyKind {
    fn default() -> DependencyKind {
        DependencyKind::Normal
    }
}

/// The `kind` can be `null`, which is interpreted as the default - `Normal`.
fn parse_dependency_kind<'de, D>(d: D) -> Result<DependencyKind, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

#[derive(Clone, Deserialize, Debug)]
/// A dependency of the main crate
pub struct Dependency {
    /// Name as given in the `Cargo.toml`
    pub name: String,
    source: Option<String>,
    /// The required version
    pub req: VersionReq,
    /// The kind of dependency this is
    #[serde(deserialize_with = "parse_dependency_kind")]
    pub kind: DependencyKind,
    /// Whether this is required or optional
    optional: bool,
    uses_default_features: bool,
    features: Vec<String>,
    target: Option<String>,
}
