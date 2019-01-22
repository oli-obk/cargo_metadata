#![deny(missing_docs)]
//! Structured access to the output of `cargo metadata` and `cargo --message-format=json`.
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
//! let mut cmd = cargo_metadata::MetadataCommand::new();
//! let manifest_path = match args.next() {
//!     Some(ref p) if p == "--manifest-path" => {
//!         cmd.manifest_path(args.next().unwrap());
//!     }
//!     Some(p) => {
//!         cmd.manifest_path(p.trim_left_matches("--manifest-path="));
//!     }
//!     None => {}
//! };
//!
//! let _metadata = cmd.exec().unwrap();
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
//! let mut cmd = cargo_metadata::MetadataCommand::new();
//! if let Some(path) = args.arg_manifest_path {
//!     cmd.manifest_path(path);
//! }
//! let _metadata = cmd.exec().unwrap();
//! # }
//! ```
//!
//! With [`clap`](https://docs.rs/clap):
//!
//! ```rust
//! # // This should be kept in sync with the equivalent example in the readme.
//! # extern crate cargo_metadata;
//! # extern crate clap;
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
//! let mut cmd = cargo_metadata::MetadataCommand::new();
//! if let Some(path) = matches.value_of("manifest-path") {
//!     cmd.manifest_path(path);
//! }
//! let _metadata = cmd.exec().unwrap();
//! ```
//! With [`structopt`](https://docs.rs/structopt):
//!
//! ```rust
//! # // This should be kept in sync with the equivalent example in the readme.
//! # extern crate cargo_metadata;
//! # #[macro_use] extern crate structopt;
//! # use std::path::PathBuf;
//! # use structopt::StructOpt;
//! # fn main() {
//! #[derive(Debug, StructOpt)]
//! struct Opt {
//!     #[structopt(name = "PATH", long="manifest-path", parse(from_os_str))]
//!     manifest_path: Option<PathBuf>,
//! }
//!
//! let opt = Opt::from_args();
//! let mut cmd = cargo_metadata::MetadataCommand::new();
//! if let Some(path) = opt.manifest_path {
//!     cmd.manifest_path(path);
//! }
//! let _metadata = cmd.exec().unwrap();
//! # }
//! ```
//!
//! Pass features flags
//!
//! ```rust
//! # // This should be kept in sync with the equivalent example in the readme.
//! # extern crate cargo_metadata;
//! # use std::path::Path;
//! # fn main() {
//! use cargo_metadata::{MetadataCommand, CargoOpt};
//!
//! let _metadata = MetadataCommand::new()
//!     .manifest_path("./Cargo.toml")
//!     .features(CargoOpt::AllFeatures)
//!     .exec()
//!     .unwrap();
//! # }
//! ```
//!
//! Parse message-format output:
//!
//! ```
//! # extern crate cargo_metadata;
//! use std::process::{Stdio, Command};
//! use cargo_metadata::Message;
//! let command = Command::new("cargo")
//!     .args(&["build", "--message-format=json"])
//!     .stdout(Stdio::piped())
//!     .spawn().unwrap();
//!
//! for message in cargo_metadata::parse_message_stream(command.stdout.unwrap()) {
//!     match message.unwrap() {
//!         Message::CompilerMessage(msg) => {
//!             println!("{:?}", msg);
//!         },
//!         Message::CompilerArtifact(artifact) => {
//!             println!("{:?}", artifact);
//!         },
//!         Message::BuildScriptExecuted(script) => {
//!             println!("{:?}", script);
//!         },
//!         _ => () // Unknown message
//!     }
//! }
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
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::from_utf8;
use std::io::Read;

use semver::Version;

pub use dependency::{Dependency, DependencyKind};
pub use errors::{Error, ErrorKind, Result};
pub use dependency::{Dependency, DependencyKind};
pub use diagnostic::*;

mod dependency;
mod errors;

/// An "opaque" identifier for a package.
/// It is possible to inspect the `repr` field, if the need arises, but its
/// precise format is an implementation detail and is subject to change.
///
/// `Metadata` can be indexed by `PackageId`.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct PackageId {
    /// The underlying string representation of id.
    pub repr: String,
}

impl std::fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.repr, f)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
/// Starting point for metadata returned by `cargo metadata`
pub struct Metadata {
    /// A list of all crates referenced by this crate (and the crate itself)
    pub packages: Vec<Package>,
    /// A list of all workspace members
    pub workspace_members: Vec<PackageId>,
    /// Dependencies graph
    pub resolve: Option<Resolve>,
    /// Workspace root
    pub workspace_root: PathBuf,
    /// Build directory
    pub target_directory: PathBuf,
    version: usize,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

impl<'a> std::ops::Index<&'a PackageId> for Metadata {
    type Output = Package;

    fn index(&self, idx: &'a PackageId) -> &Package {
        self.packages
            .iter()
            .find(|p| p.id == *idx)
            .unwrap_or_else(|| panic!("no package with this id: {:?}", idx))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
/// A dependency graph
pub struct Resolve {
    /// Nodes in a dependencies graph
    pub nodes: Vec<Node>,

    /// The crate for which the metadata was read.
    pub root: Option<PackageId>,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
/// A node in a dependencies graph
pub struct Node {
    /// An opaque identifier for a package
    pub id: PackageId,
    /// Dependencies in a structured format.
    ///
    /// `deps` handles renamed dependencies whereas `dependencies` does not.
    #[serde(default)]
    pub deps: Vec<NodeDep>,

    /// List of opaque identifiers for this node's dependencies.
    /// It doesn't support renamed dependencies. See `deps`.
    pub dependencies: Vec<PackageId>,

    /// Features enabled on the crate
    #[serde(default)]
    pub features: Vec<String>,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
/// A dependency in a node
pub struct NodeDep {
    /// The name of the dependency's library target.
    /// If the crate was renamed, it is the new name.
    pub name: String,
    /// Package ID (opaque unique identifier)
    pub pkg: PackageId,
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
    pub version: Version,
    /// Authors given in the `Cargo.toml`
    #[serde(default)]
    pub authors: Vec<String>,
    /// An opaque identifier for a package
    pub id: PackageId,
    source: Option<String>,
    /// Description as given in the `Cargo.toml`
    pub description: Option<String>,
    /// List of dependencies of this particular package
    pub dependencies: Vec<Dependency>,
    /// License as given in the `Cargo.toml`
    pub license: Option<String>,
    /// If the package is using a nonstandard license, this key may be specified instead of
    /// `license`, and must point to a file relative to the manifest.
    pub license_file: Option<PathBuf>,
    /// Targets provided by the crate (lib, bin, example, test, ...)
    pub targets: Vec<Target>,
    /// Features provided by the crate, mapped to the features required by that feature.
    pub features: HashMap<String, Vec<String>>,
    /// Path containing the `Cargo.toml`
    pub manifest_path: PathBuf,
    /// Categories as given in the `Cargo.toml`
    #[serde(default)]
    pub categories: Vec<String>,
    /// Keywords as given in the `Cargo.toml`
    #[serde(default)]
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
    /// The name of a native library the package is linking to.
    pub links: Option<String>,
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

    #[serde(default)]
    #[serde(rename = "required-features")]
    /// This target is built only if these features are enabled.
    /// It doesn't apply to `lib` targets.
    pub required_features: Vec<String>,
    /// Path to the main source file of the target
    pub src_path: PathBuf,
    /// Rust edition for this target
    #[serde(default = "edition_default")]
    pub edition: String,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

fn edition_default() -> String {
    "2015".to_string()
}

/// Cargo features flags
#[derive(Debug, Clone)]
pub enum CargoOpt {
    /// Run cargo with `--features-all`
    AllFeatures,
    /// Run cargo with `--no-default-features`
    NoDefaultFeatures,
    /// Run cargo with `--features <FEATURES>`
    SomeFeatures(Vec<String>),
}

/// Profile settings used to determine which compiler flags to use for a
/// target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactProfile {
    /// Optimization level. Possible values are 0-3, s or z.
    pub opt_level: String,
    /// The amount of debug info. 0 for none, 1 for limited, 2 for full
    pub debuginfo: Option<u32>,
    /// State of the `cfg(debug_assertions)` directive, enabling macros like
    /// `debug_assert!`
    pub debug_assertions: bool,
    /// State of the overflow checks.
    pub overflow_checks: bool,
    /// Whether this profile is a test
    pub test: bool,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

/// A compiler-generated file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// The workspace member this artifact belongs to
    pub package_id: WorkspaceMember,
    /// The target this artifact was compiled for
    pub target: Target,
    /// The profile this artifact was compiled with
    pub profile: ArtifactProfile,
    /// The enabled features for this artifact
    pub features: Vec<String>,
    /// The full paths to the generated artifacts
    pub filenames: Vec<String>,
    /// If true, then the files were already generated
    pub fresh: bool,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

/// Message left by the compiler
// TODO: structify message
// TODO: Better name. This one comes from machine_message.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FromCompiler {
    /// The workspace member this message belongs to
    pub package_id: WorkspaceMember,
    /// The target this message is aimed at
    pub target: Target,
    /// The message the compiler sent.
    pub message: diagnostic::Diagnostic,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

/// Output of a Build Script execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildScript {
    /// The workspace member this build script execution belongs to
    pub package_id: WorkspaceMember,
    /// The libs to link
    pub linked_libs: Vec<String>,
    /// The paths to search when resolving libs
    pub linked_paths: Vec<String>,
    /// The paths to search when resolving libs
    pub cfgs: Vec<String>,
    /// The environment variables to add to the compilation
    pub env: Vec<(String, String)>,
    #[doc(hidden)]
    #[serde(skip)]
    __do_not_match_exhaustively: (),
}

/// A cargo message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
pub enum Message {
    /// The compiler generated an artifact
    CompilerArtifact(Artifact),
    /// The compiler wants to display a message
    CompilerMessage(FromCompiler),
    /// A build script successfully executed.
    BuildScriptExecuted(BuildScript),
    #[doc(hidden)]
    #[serde(other)]
    Unknown
}

/// Obtain metadata only about the root package and don't fetch dependencies
///
/// # Parameters
///
/// - `manifest_path`: Path to the manifest.
pub fn metadata(manifest_path: Option<&Path>) -> Result<Metadata> {
    metadata_run(manifest_path, false, None)
/// A builder for configurating `cargo metadata` invocation.
#[derive(Debug, Clone, Default)]
pub struct MetadataCommand {
    manifest_path: Option<PathBuf>,
    current_dir: Option<PathBuf>,
    no_deps: bool,
    features: Option<CargoOpt>,
}

impl MetadataCommand {
    /// Creates a default `cargo metadata` command, which will look for
    /// `Cargo.toml` in the ancestors of the current directory.
    pub fn new() -> MetadataCommand {
        MetadataCommand::default()
    }
    /// Path to `Cargo.toml`
    pub fn manifest_path(&mut self, path: impl AsRef<Path>) -> &mut MetadataCommand {
        self.manifest_path = Some(path.as_ref().to_path_buf());
        self
    }
    /// Current directory of the `cargo metadata` process.
    pub fn current_dir(&mut self, path: impl AsRef<Path>) -> &mut MetadataCommand {
        self.current_dir = Some(path.as_ref().to_path_buf());
        self
    }
    /// Output information only about the root package and don't fetch dependencies.
    pub fn no_deps(&mut self) -> &mut MetadataCommand {
        self.no_deps = true;
        self
    }
    /// Which features to include.
    pub fn features(&mut self, features: CargoOpt) -> &mut MetadataCommand {
        self.features = Some(features);
        self
    }
    /// Runs configured `cargo metadata` and returns parsed `Metadata`.
    pub fn exec(&mut self) -> Result<Metadata> {
        let cargo = env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
        let mut cmd = Command::new(cargo);
        cmd.args(&["metadata", "--format-version", "1"]);

        if self.no_deps {
            cmd.arg("--no-deps");
        }

        if let Some(features) = &self.features {
            match features {
                CargoOpt::AllFeatures => cmd.arg("--all-features"),
                CargoOpt::NoDefaultFeatures => cmd.arg("--no-default-features"),
                CargoOpt::SomeFeatures(ftrs) => cmd.arg(format!("--features {:?}", ftrs)),
            };
        }

        if let Some(manifest_path) = &self.manifest_path {
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
}

/// An iterator of Message.
type MessageIterator<R> = serde_json::StreamDeserializer<'static, serde_json::de::IoRead<R>, Message>;

// TODO: Should I use de::Read instead, to support deserializing from a slice?
/// Creates an iterator of Message from a Read outputting a stream of JSON
/// messages. For usuage information, look at the top-level documentation.
pub fn parse_message_stream<R: Read>(input: R) -> MessageIterator<R> {
    serde_json::Deserializer::from_reader(input).into_iter::<Message>()
}
