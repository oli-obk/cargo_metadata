#![deny(missing_docs)]
//! Structured access to the output of `cargo metadata`
//! Usually used from within a `cargo-*` executable
//!
//! ## Examples
//!
//! With [`std::env::args()`](https://doc.rust-lang.org/std/env/fn.args.html):
//!
//! ```rust
//! # // This should be kept in sync with the equivalent example in the readme.
//! # extern crate cargo_metadata;
//! # use std::path::Path;
//! let mut args = std::env::args().skip_while(|val| !val.starts_with("--manifest-path"));
//!
//! let manifest_path = match args.next() {
//!     Some(ref p) if p == "--manifest-path" => args.next(),
//!     Some(p) => Some(p.trim_left_matches("--manifest-path=").to_string()),
//!     None => None,
//! };
//!
//! let _metadata = cargo_metadata::metadata(manifest_path.as_ref().map(Path::new)).unwrap();
//! ```
//!
//! With [`docopt`](https://docs.rs/docopt):
//!
//! ```rust
//! # // This should be kept in sync with the equivalent example in the readme.
//! # extern crate cargo_metadata;
//! # extern crate docopt;
//! # #[macro_use] extern crate serde_derive;
//! # use std::path::Path;
//! # use docopt::Docopt;
//! # fn main() {
//! const USAGE: &str = "
//!     Cargo metadata test function
//!
//!     Usage:
//!       cargo_metadata [--manifest-path PATH]
//! ";
//!
//! #[derive(Debug, Deserialize)]
//! struct Args {
//!     arg_manifest_path: Option<String>,
//! }
//!
//! let args: Args = Docopt::new(USAGE)
//!     .and_then(|d| d.deserialize())
//!     .unwrap_or_else(|e| e.exit());
//!
//! let _metadata =
//!     cargo_metadata::metadata(args.arg_manifest_path.as_ref().map(Path::new)).unwrap();
//! # }
//! ```
//!
//! With [`clap`](https://docs.rs/clap):
//!
//! ```rust
//! # // This should be kept in sync with the equivalent example in the readme.
//! # extern crate cargo_metadata;
//! # extern crate clap;
//! # use std::path::Path;
//!
//! let matches = clap::App::new("myapp")
//!     .arg(
//!         clap::Arg::with_name("manifest-path")
//!             .long("manifest-path")
//!             .value_name("PATH")
//!             .takes_value(true),
//!     )
//!     .get_matches();
//!
//! let _metadata =
//!     cargo_metadata::metadata(matches.value_of("manifest-path").map(Path::new)).unwrap();
//! ```
//!
//! Pass features flags
//!
//! ```rust
//! # // This should be kept in sync with the equivalent example in the readme.
//! # extern crate cargo_metadata;
//! # use std::path::Path;
//!
//! let manifest_path = Path::new("./Cargo.toml");
//! let features = cargo_metadata::CargoOpt::AllFeatures;

//! let _metadata =
//! cargo_metadata::metadata_run(Some(manifest_path), false, Some(features)).unwrap();
//!
//! ```

#[macro_use]
extern crate error_chain;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

pub use errors::{Error, ErrorKind, Result};
pub use dependency::{Dependency, DependencyKind};

mod errors;
mod dependency;

#[derive(Clone, Serialize, Deserialize, Debug)]
/// Starting point for metadata returned by `cargo metadata`
pub struct Metadata {
    /// A list of all crates referenced by this crate (and the crate itself)
    pub packages: Vec<Package>,
    /// A list of all workspace members
    pub workspace_members: Vec<WorkspaceMember>,
    /// Dependencies graph
    pub resolve: Option<Resolve>,
    /// Workspace root
    #[serde(default)]
    pub workspace_root: String,
    /// Build directory
    pub target_directory: String,
    version: usize,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
/// A dependency graph
pub struct Resolve {
    /// Nodes in a dependencies graph
    pub nodes: Vec<Node>,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
/// A node in a dependencies graph
pub struct Node {
    /// An opaque identifier for a package
    pub id: String,
    /// List of opaque identifiers for this node's dependencies
    pub dependencies: Vec<String>,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
/// A crate
pub struct Package {
    /// Name as given in the `Cargo.toml`
    pub name: String,
    /// Version given in the `Cargo.toml`
    pub version: String,
    /// Authors given in the `Cargo.toml`
    #[serde(default)]
    pub authors: Vec<String>,
    /// An opaque identifier for a package
    pub id: String,
    source: Option<String>,
    /// Description as given in the `Cargo.toml`
    pub description: Option<String>,
    /// List of dependencies of this particular package
    pub dependencies: Vec<Dependency>,
    /// License as given in the `Cargo.toml`
    pub license: Option<String>,
    /// Targets provided by the crate (lib, bin, example, test, ...)
    pub targets: Vec<Target>,
    /// Features provided by the crate, mapped to the features required by that feature.
    pub features: HashMap<String, Vec<String>>,
    /// Path containing the `Cargo.toml`
    pub manifest_path: String,
    /// Categories as given in the `Cargo.toml`
    pub categories: Vec<String>,
    /// Keywords as given in the `Cargo.toml`
    pub keywords: Vec<String>,
    /// Readme as given in the `Cargo.toml`
    pub readme: Option<String>,
    /// Repository as given in the `Cargo.toml`
    pub repository: Option<String>,
    /// Default Rust edition for the package
    ///
    /// Beware that individual targets may specify their own edition in
    /// [`Target::edition`](struct.Target.html#structfield.edition).
    #[serde(default = "edition_default")]
    pub edition: String,
    /// Contents of the free form package.metadata section
    ///
    /// This contents can be serialized to a struct using serde:
    ///
    /// ```rust
    /// #[macro_use]
    /// extern crate serde_json;
    /// #[macro_use]
    /// extern crate serde_derive;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct SomePackageMetadata {
    ///     some_value: i32,
    /// }
    ///
    /// fn main() {
    ///     let value = json!({
    ///         "some_value": 42,
    ///     });
    ///
    ///     let package_metadata: SomePackageMetadata = serde_json::from_value(value).unwrap();
    ///     assert_eq!(package_metadata.some_value, 42);
    /// }
    ///
    /// ```
    #[serde(default)]
    pub metadata: serde_json::Value,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
/// A single target (lib, bin, example, ...) provided by a crate
pub struct Target {
    /// Name as given in the `Cargo.toml` or generated from the file name
    pub name: String,
    /// Kind of target ("bin", "example", "test", "bench", "lib")
    pub kind: Vec<String>,
    /// Almost the same as `kind`, except when an example is a library instad of an executable.
    /// In that case `crate_types` contains things like `rlib` and `dylib` while `kind` is `example`
    #[serde(default)]
    pub crate_types: Vec<String>,
    /// Path to the main source file of the target
    pub src_path: String,
    /// Rust edition for this target
    #[serde(default = "edition_default")]
    pub edition: String,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// A workspace member. This is basically identical to `cargo::core::package_id::PackageId`, except
/// that this does not use `Arc` internally.
#[serde(transparent)]
pub struct WorkspaceMember {
    /// The raw package id as given by cargo
    pub raw: String,
}

impl WorkspaceMember {
    fn part(&self, n: usize) -> &str {
        self.raw.splitn(3, ' ').nth(n).unwrap()
    }
    /// The name of the crate
    pub fn name(&self) -> &str {
        self.part(0)
    }
    /// The version of the crate
    pub fn version(&self) -> semver::Version {
        semver::Version::parse(self.part(1)).expect("bad version in cargo metadata")
    }
    /// The path to the crate in url format
    pub fn url(&self) -> &str {
        let url = self.part(2);
        &url[1..url.len() - 1]
    }
}

fn edition_default() -> String {
    "2015".to_string()
}

/// Cargo features flags
pub enum CargoOpt {
    /// Run cargo with `--features-all`
    AllFeatures,
    /// Run cargo with `--no-default-features`
    NoDefaultFeatures,
    /// Run cargo with `--features <FEATURES>`
    SomeFeatures(Vec<String>),
}

/// Obtain metadata only about the root package and don't fetch dependencies
///
/// # Parameters
///
/// - `manifest_path`: Path to the manifest.
pub fn metadata(manifest_path: Option<&Path>) -> Result<Metadata> {
    metadata_run(manifest_path, false, None)
}

/// Obtain metadata only about the root package and dependencies
///
/// # Parameters
///
/// - `manifest_path`: Path to the manifest.
/// - `deps`: Whether to include dependencies.
pub fn metadata_deps(manifest_path: Option<&Path>, deps: bool) -> Result<Metadata> {
    metadata_run(manifest_path, deps, None)
}

/// The main entry point to obtaining metadata
///
/// # Parameters
///
/// - `manifest_path`: Path to the manifest.
/// - `deps`: Whether to include dependencies.
/// - `feat`: Which features to include, `None` for defaults.
pub fn metadata_run(manifest_path: Option<&Path>, deps: bool, features: Option<CargoOpt>) -> Result<Metadata> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    let mut cmd = Command::new(cargo);
    cmd.arg("metadata");

    if !deps {
        cmd.arg("--no-deps");
    }

    if let Some(features) = features {
        match features {
            CargoOpt::AllFeatures => cmd.arg("--all-features"),
            CargoOpt::NoDefaultFeatures => cmd.arg("--no-default-features"),
            CargoOpt::SomeFeatures(ftrs) => cmd.arg(format!("--fatures {:?}", ftrs)),
        };
    }

    cmd.args(&["--format-version", "1"]);
    if let Some(manifest_path) = manifest_path {
        cmd.arg("--manifest-path").arg(manifest_path.as_os_str());
    }
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(ErrorKind::CargoMetadata(String::from_utf8(output.stderr)?).into());
    }
    let stdout = from_utf8(&output.stdout)?;
    let meta = serde_json::from_str(stdout)?;
    Ok(meta)
}
