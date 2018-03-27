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
use std::fmt::Write;

pub use errors::{Error, ErrorKind, Result};
pub use dependency::{Dependency, DependencyKind};
use serde::{de, ser, Serializer};

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
    /// An opaque identifier for a package
    pub id: String,
    source: Option<String>,
    /// List of dependencies of this particular package
    pub dependencies: Vec<Dependency>,
    /// Targets provided by the crate (lib, bin, example, test, ...)
    pub targets: Vec<Target>,
    /// Features provided by the crate, mapped to the features required by that feature.
    pub features: HashMap<String, Vec<String>>,
    /// Path containing the `Cargo.toml`
    pub manifest_path: String,
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
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

#[derive(Clone, Debug)]
/// A workspace member. This is basically identical to `cargo::core::package_id::PackageId`, expect
/// that this does not use `Arc` internally.
pub struct WorkspaceMember {
    /// A name of workspace member.
    pub name: String,
    /// A version of workspace member.
    pub version: semver::Version,
    /// A source id of workspace member.
    pub url: String,
    #[doc(hidden)] __do_not_match_exhaustively: (),
}

impl<'de> de::Deserialize<'de> for WorkspaceMember {
    fn deserialize<D>(d: D) -> std::result::Result<WorkspaceMember, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let string = String::deserialize(d)?;
        let mut s = string.splitn(3, ' ');
        let name = s.next().unwrap();
        let version = s.next().unwrap();
        let version = semver::Version::parse(version).map_err(de::Error::custom)?;
        let url = &s.next().unwrap();
        let url = &url[1..url.len() - 1];
        Ok(WorkspaceMember {
            name: name.to_owned(),
            version,
            url: url.to_owned(),
            __do_not_match_exhaustively: (),
        })
    }
}

impl ser::Serialize for WorkspaceMember {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut output = String::new();
        write!(output, "{} {} ({})", self.name, self.version, self.url).unwrap();
        serializer.serialize_str(&output)
    }
}

/// Obtain metadata only about the root package and don't fetch dependencies
///
/// # Parameters
///
/// - `manifest_path`: Path to the manifest.
pub fn metadata(manifest_path: Option<&Path>) -> Result<Metadata> {
    metadata_deps(manifest_path, false)
}

/// The main entry point to obtaining metadata
///
/// # Parameters
///
/// - `manifest_path`: Path to the manifest.
/// - `deps`: Whether to include dependencies.
pub fn metadata_deps(manifest_path: Option<&Path>, deps: bool) -> Result<Metadata> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    let mut cmd = Command::new(cargo);
    cmd.arg("metadata");

    if !deps {
        cmd.arg("--no-deps");
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
