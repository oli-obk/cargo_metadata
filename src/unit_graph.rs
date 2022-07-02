use cargo_platform::Platform;
use serde::{Deserialize, Serialize};

use crate::{PackageId, Target};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UnitGraph {
    /// Version of the JSON output structure.
    /// If any backwards incompatible changes are made, this value will be increased.
    version: usize,
    /// Array of all build units.
    pub units: Vec<Unit>,
    /// Array of indices in the "units" array that are the "roots" of the dependency graph.
    pub roots: Vec<usize>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Unit {
    /// An opaque string which indicates the package.
    pub pkg_id: PackageId,
    /// The Cargo target
    pub target: Target,
    /// The profile settings for this unit.
    /// These values may not match the profile defined in the manifest.
    /// Units can use modified profile settings. For example, the "panic"
    /// setting can be overridden for tests to force it to "unwind".
    pub profile: Profile,
    /// Which platform this target is being built for.
    /// A value of `None` indicates it is for the host.
    pub platform: Option<Platform>,
    /// The "mode" for this unit.
    pub mode: Mode,
    /// Array of features enabled on this unit.
    pub features: Vec<String>,
    /// Whether or not this is a standard-library unit,
    /// part of the unstable build-std feature
    #[serde(default)]
    pub is_std: bool,
    /// Array of dependencies of this unit.
    pub dependencies: Vec<Dependency>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Profile {
    /// The profile name these settings are derived from.
    pub name: String, // TODO: could be an enum.
    /// The optimization level.
    pub opt_level: String, // TODO: could be an enum.
    /// The LTO setting.
    pub lto: String, // TODO: Almost definitely could be an enum
    /// The codegen units as an integer.
    /// `None` if it should use the compiler's default.
    pub codegen_units: Option<u32>,
    /// The debug information level as an integer.
    /// `None` if it should use the compiler's default (0).
    pub debuginfo: Option<u32>,
    /// Whether or not debug-assertions are enabled.
    pub debug_assertions: bool,
    /// Whether or not overflow-checks are enabled.
    pub overflow_checks: bool,
    /// Whether or not incremental is enabled.
    pub rpath: bool,
    /// Whether or not incremental is enabled.
    pub incremental: bool,
    /// The panic strategy.
    pub panic: PanicStrategy,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum PanicStrategy {
    #[serde(rename = "unwind")]
    Unwind,
    #[serde(rename = "abort")]
    Abort,
}

/// The "mode" of a unit.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Mode {
    /// Build using `rustc` as a test.
    #[serde(rename = "test")]
    Test,
    /// Build using `rustc`.
    #[serde(rename = "build")]
    Build,
    /// Build using `rustc` in "check" mode.
    #[serde(rename = "check")]
    Check,
    /// Build using `rustdoc`.
    #[serde(rename = "doc")]
    Doc,
    /// Test using `rustdoc`.
    #[serde(rename = "doctest")]
    Doctest,
    /// Represents the execution of a build script.
    #[serde(rename = "run-custom-build")]
    RunCustomBuild,
}

/// Array of dependencies of a unit.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Dependency {
    /// Index in the "units" array for the dependency.
    pub index: usize,
    /// The name that this dependency will be referred as.
    pub extern_crate_name: String,
    /// Whether or not this dependency is "public",
    /// part of the unstable public-dependency feature.
    /// If `None` the public-dependency feature is not enabled.
    pub public: Option<bool>,
    /// Whether or not this dependency is injected into the prelude,
    /// currently used by the build-std feature.
    #[serde(default)]
    pub noprelude: bool,
}
