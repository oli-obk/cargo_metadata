extern crate cargo_metadata;
extern crate semver;

use std::path::Path;

use semver::Version;

#[test]
fn metadata() {
    let metadata = cargo_metadata::metadata(None).unwrap();

    assert_eq!(metadata.packages[0].name, "cargo_metadata");
    assert_eq!(metadata.packages[0].targets.len(), 2);

    assert_eq!(metadata.packages[0].targets[0].name, "cargo_metadata");
    assert_eq!(metadata.packages[0].targets[0].kind[0], "lib");
    assert_eq!(metadata.packages[0].targets[0].crate_types[0], "lib");

    assert_eq!(metadata.packages[0].targets[1].name, "selftest");
    assert_eq!(metadata.packages[0].targets[1].kind[0], "test");
    assert_eq!(metadata.packages[0].targets[1].crate_types[0], "bin");
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
