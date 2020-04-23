extern crate cargo_metadata;
extern crate semver;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::env::{set_current_dir, self};
use std::path::{Path, PathBuf, Component::CurDir};

use semver::Version;

use cargo_metadata::{CargoOpt, Error, MetadataCommand};

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct TestPackageMetadata {
    some_field: bool,
    other_field: String,
}

fn chdir_find_manifest() -> (PathBuf, PathBuf) {
    // 1. defuses any chdir that cargo has done
    //    (see https://github.com/rust-lang/cargo/issues/8148)
    //  2. returns the manifest path for passing to builder etc.

    // This is idempotent (which is important, since cargo might
    // call any subset of our test functions in whatever order)
    // and also threadsafe.

    let manifest_dir : PathBuf =
        env::var_os("CARGO_MANIFEST_DIR")
        .as_ref()
        .map(|v| Path::new(v))
        .unwrap_or(CurDir.as_ref())
        .to_owned();

    if let Some(cargo_home) = env::var_os("CARGO_HOME") {
        set_current_dir(cargo_home)
            .expect("chdir to undo cargo's chdir to manifest dir");
        // Best we can until https://github.com/rust-lang/cargo/issues/8148
    }

    let mut manifest_path = manifest_dir.clone();
    manifest_path.push("Cargo.toml");

    (manifest_dir, manifest_path)
}

#[test]
fn metadata() {
    let (manifest_dir, manifest_path) = chdir_find_manifest();
    let metadata = MetadataCommand::new().no_deps()
        .manifest_path(&manifest_path).exec().unwrap();

    assert_eq!(
        manifest_dir.join("target"),
        Path::new(&metadata.target_directory)
    );

    let this = &metadata.packages[0];
    assert_eq!(this.name, "cargo_metadata");
    assert_eq!(this.targets.len(), 3);

    let lib = this
        .targets
        .iter()
        .find(|t| t.name == "cargo_metadata")
        .unwrap();
    assert_eq!(lib.kind[0], "lib");
    assert_eq!(lib.crate_types[0], "lib");

    let selftest = this.targets.iter().find(|t| t.name == "selftest").unwrap();
    assert_eq!(selftest.name, "selftest");
    assert_eq!(selftest.kind[0], "test");
    assert_eq!(selftest.crate_types[0], "bin");

    let package_metadata = &metadata.packages[0]
        .metadata
        .as_object()
        .expect("package.metadata must be a table.");
    assert_eq!(package_metadata.len(), 1);

    let value = package_metadata.get("cargo_metadata_test").unwrap();
    let test_package_metadata: TestPackageMetadata = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(
        test_package_metadata,
        TestPackageMetadata {
            some_field: true,
            other_field: "foo".into(),
        }
    );
}

#[test]
fn builder_interface() {
    let (manifest_dir, manifest_path) = chdir_find_manifest();
    let manifest_path : &str = manifest_path.to_str()
        .expect("manifest path should be utf8");
    let _ = MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()
        .unwrap();
    let _ = MetadataCommand::new()
        .manifest_path(String::from(manifest_path))
        .exec()
        .unwrap();
    let _ = MetadataCommand::new()
        .manifest_path(PathBuf::from(manifest_path))
        .exec()
        .unwrap();
    let _ = MetadataCommand::new()
        .manifest_path(manifest_path)
        .no_deps()
        .exec()
        .unwrap();
    let _ = MetadataCommand::new()
        .manifest_path(manifest_path)
        .features(CargoOpt::AllFeatures)
        .exec()
        .unwrap();
    let _ = MetadataCommand::new()
        .manifest_path(manifest_path)
        .current_dir(manifest_dir)
        .exec()
        .unwrap();
}

#[test]
fn error1() {
    match MetadataCommand::new().manifest_path("foo").exec() {
        Err(Error::CargoMetadata { stderr }) => assert_eq!(
            stderr.trim(),
            "error: the manifest-path must be a path to a Cargo.toml file"
        ),
        _ => unreachable!(),
    }
}

#[test]
fn error2() {
    match MetadataCommand::new()
        .manifest_path("foo/Cargo.toml")
        .exec()
    {
        Err(Error::CargoMetadata { stderr }) => assert_eq!(
            stderr.trim(),
            "error: manifest path `foo/Cargo.toml` does not exist"
        ),
        _ => unreachable!(),
    }
}

#[test]
fn cargo_path() {
    match MetadataCommand::new()
        .cargo_path("this does not exist")
        .exec()
    {
        Err(Error::Io(e)) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        _ => unreachable!(),
    }
}

#[test]
fn metadata_deps() {
    std::env::set_var("CARGO_PROFILE", "3");
    let (_manifest_dir, manifest_path) = chdir_find_manifest();
    let metadata = MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()
        .unwrap();
    let this_id = metadata
        .workspace_members
        .first()
        .expect("Did not find ourselves");
    let this = &metadata[this_id];

    assert_eq!(this.name, "cargo_metadata");

    let lib = this
        .targets
        .iter()
        .find(|t| t.name == "cargo_metadata")
        .unwrap();
    assert_eq!(lib.kind[0], "lib");
    assert_eq!(lib.crate_types[0], "lib");

    let selftest = this.targets.iter().find(|t| t.name == "selftest").unwrap();
    assert_eq!(selftest.name, "selftest");
    assert_eq!(selftest.kind[0], "test");
    assert_eq!(selftest.crate_types[0], "bin");

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
