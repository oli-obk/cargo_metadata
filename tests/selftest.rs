extern crate cargo_metadata;
extern crate semver;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::env::current_dir;
use std::path::Path;
use std::env;

use semver::Version;

use cargo_metadata::{Error, ErrorKind};

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct TestPackageMetadata {
    some_field: bool,
    other_field: String,
}

#[test]
fn metadata() {
    let metadata = cargo_metadata::metadata(None).unwrap();

    assert_eq!(
        current_dir().unwrap().join("target"),
        Path::new(&metadata.target_directory)
    );

    assert_eq!(metadata.packages[0].name, "cargo_metadata");
    assert_eq!(metadata.packages[0].targets.len(), 2);

    assert_eq!(metadata.packages[0].targets[0].name, "cargo_metadata");
    assert_eq!(metadata.packages[0].targets[0].kind[0], "lib");
    assert_eq!(metadata.packages[0].targets[0].crate_types[0], "lib");

    assert_eq!(metadata.packages[0].targets[1].name, "selftest");
    assert_eq!(metadata.packages[0].targets[1].kind[0], "test");
    assert_eq!(metadata.packages[0].targets[1].crate_types[0], "bin");

    // Hack until the package metadata field reaches the stable channel (in version 1.27).
    if env::var("TRAVIS_RUST_VERSION") != Ok("stable".into()) {
        let package_metadata = &metadata.packages[0].metadata.as_object()
            .expect("package.metadata must be a table. \
            NOTE: This test currently only works on the beta and nightly channel.");
        assert_eq!(package_metadata.len(), 1);

        let value = package_metadata.get("cargo_metadata_test").unwrap();
        let test_package_metadata: TestPackageMetadata = serde_json::from_value(value.clone())
            .unwrap();
        assert_eq!(test_package_metadata, TestPackageMetadata {
            some_field: true,
            other_field: "foo".into(),
        });
    }
}

#[test]
fn error1() {
    match cargo_metadata::metadata_deps(Some(Path::new("foo")), true) {
        Err(Error(ErrorKind::CargoMetadata(s), _)) => assert_eq!(
            s.trim(),
            "error: the manifest-path must be a path to a Cargo.toml file"
        ),
        _ => unreachable!(),
    }
}

#[test]
fn error2() {
    match cargo_metadata::metadata_deps(Some(Path::new("foo/Cargo.toml")), true) {
        Err(Error(ErrorKind::CargoMetadata(s), _)) => assert_eq!(
            s.trim(),
            "error: manifest path `foo/Cargo.toml` does not exist"
        ),
        _ => unreachable!(),
    }
}

#[test]
fn metadata_deps() {
    let metadata = cargo_metadata::metadata_deps(Some(Path::new("Cargo.toml")), true).unwrap();
    let this = metadata
        .packages
        .iter()
        .find(|package| package.name == "cargo_metadata")
        .expect("Did not find ourselves");

    assert_eq!(this.name, "cargo_metadata");
    assert_eq!(this.targets.len(), 2);

    assert_eq!(this.targets[0].name, "cargo_metadata");
    assert_eq!(this.targets[0].kind[0], "lib");
    assert_eq!(this.targets[0].crate_types[0], "lib");

    assert_eq!(this.targets[1].name, "selftest");
    assert_eq!(this.targets[1].kind[0], "test");
    assert_eq!(this.targets[1].crate_types[0], "bin");

    let dependencies = &this.dependencies;

    let serde = dependencies
        .iter()
        .find(|dep| dep.name == "serde")
        .expect("Did not find serde dependency");

    assert_eq!(serde.kind, cargo_metadata::DependencyKind::Normal);
    assert!(!serde.req.matches(&Version::parse("1.0.0").unwrap()));
    assert!(serde.req.matches(&Version::parse("1.99.99").unwrap()));
    assert!(!serde.req.matches(&Version::parse("2.0.0").unwrap()));
}

#[test]
fn workspace_member_serialization_deserialization() {
    let original =
        "\"security-framework 0.1.16 (registry+https://github.com/rust-lang/crates.io-index)\"";
    let member: cargo_metadata::WorkspaceMember = serde_json::from_str(original).unwrap();
    assert_eq!(member.name(), "security-framework");
    assert_eq!(member.version(), Version::new(0, 1, 16));
    assert_eq!(
        member.url(),
        "registry+https://github.com/rust-lang/crates.io-index"
    );

    let serialized = serde_json::to_string(&member).unwrap();
    assert_eq!(serialized, original);
}
