extern crate cargo_metadata;

#[test]
fn foo() {
    let metadata = cargo_metadata::metadata(None).unwrap();
    assert_eq!(metadata.packages[0].name, "cargo_metadata");
    assert_eq!(metadata.packages[0].targets.len(), 2);
    assert_eq!(metadata.packages[0].targets[0].name, "cargo_metadata");
    assert_eq!(metadata.packages[0].targets[0].kind[0], "lib");
    assert_eq!(metadata.packages[0].targets[1].kind[0], "test");
    assert_eq!(metadata.packages[0].targets[0].crate_types[0], "lib");
    assert_eq!(metadata.packages[0].targets[1].crate_types[0], "test");
}
