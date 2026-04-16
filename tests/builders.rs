#![cfg(feature = "builder")]

use semver::Version;

use cargo_metadata::diagnostic::{
    DiagnosticBuilder, DiagnosticCodeBuilder, DiagnosticLevel, DiagnosticSpanBuilder,
    DiagnosticSpanLineBuilder, DiagnosticSpanMacroExpansionBuilder,
};
use cargo_metadata::{
    ArtifactBuilder, ArtifactDebuginfo, ArtifactProfileBuilder, BuildFinishedBuilder,
    BuildScriptBuilder, CompilerMessageBuilder, DepKindInfoBuilder, DependencyBuilder,
    DependencyKind, FeatureName, MetadataBuilder, NodeBuilder, NodeDepBuilder, PackageBuilder,
    PackageId, ResolveBuilder, TargetBuilder, TargetKind, WorkspaceDefaultMembers,
};

#[test]
fn package_builder() {
    let pkg_id = PackageId::from("my-package 1.0.0 (path+file:///workspace)");
    let pkg = PackageBuilder::new(
        "my-package",
        Version::new(1, 0, 0),
        pkg_id.clone(),
        "/workspace/Cargo.toml",
    )
    .build()
    .unwrap();

    assert_eq!(pkg.name, "my-package");
    assert_eq!(pkg.version, Version::new(1, 0, 0));
    assert_eq!(pkg.id, pkg_id);
}

#[test]
fn dep_kind_info_builder() {
    let dep_kind_info = DepKindInfoBuilder::default()
        .kind(DependencyKind::Normal)
        .target(None)
        .build()
        .unwrap();

    assert_eq!(dep_kind_info.kind, DependencyKind::Normal);
}

#[test]
fn node_dep_builder() {
    let pkg_id = PackageId::from("pkg 1.0.0 (path+file:///workspace)");
    let dep_kind_info = DepKindInfoBuilder::default()
        .kind(DependencyKind::Normal)
        .target(None)
        .build()
        .unwrap();
    let node_dep = NodeDepBuilder::default()
        .name("dep-name")
        .pkg(pkg_id.clone())
        .dep_kinds(vec![dep_kind_info])
        .build()
        .unwrap();

    assert_eq!(node_dep.name, "dep-name");
    assert_eq!(node_dep.pkg, pkg_id);
}

#[test]
fn node_builder() {
    let pkg_id = PackageId::from("my-package 1.0.0 (path+file:///workspace)");
    let node = NodeBuilder::default()
        .id(pkg_id.clone())
        .deps(vec![])
        .dependencies(vec![])
        .features(vec![FeatureName::from("default"), FeatureName::from("std")])
        .build()
        .unwrap();

    assert_eq!(node.id, pkg_id);
    assert_eq!(node.features[0], "default");
    assert_eq!(node.features[1], "std");
}

#[test]
fn resolve_builder() {
    let pkg_id = PackageId::from("pkg 1.0.0 (path+file:///workspace)");
    let node = NodeBuilder::default()
        .id(pkg_id.clone())
        .deps(vec![])
        .dependencies(vec![])
        .features(vec![])
        .build()
        .unwrap();
    let resolve = ResolveBuilder::default()
        .nodes(vec![node])
        .root(Some(pkg_id.clone()))
        .build()
        .unwrap();

    assert_eq!(resolve.root, Some(pkg_id));
}

#[test]
fn target_builder() {
    let target = TargetBuilder::default()
        .name("my-lib")
        .kind(vec![TargetKind::from("lib")])
        .src_path("/workspace/src/lib.rs")
        .build()
        .unwrap();

    assert_eq!(target.name, "my-lib");
}

#[test]
fn metadata_builder() {
    let pkg_id = PackageId::from("my-package 1.0.0 (path+file:///workspace)");
    let pkg = PackageBuilder::new(
        "my-package",
        Version::new(1, 0, 0),
        pkg_id.clone(),
        "/workspace/Cargo.toml",
    )
    .build()
    .unwrap();
    let meta = MetadataBuilder::default()
        .packages(vec![pkg])
        .workspace_members(vec![pkg_id.clone()])
        .workspace_default_members(WorkspaceDefaultMembers::from(vec![pkg_id.clone()]))
        .resolve(None)
        .workspace_root("/workspace")
        .target_directory("/workspace/target")
        .build_directory(None)
        .workspace_metadata(serde_json::Value::Null)
        .version(2_usize)
        .build()
        .unwrap();

    assert_eq!(meta.workspace_members, vec![pkg_id]);
    assert!(meta.workspace_default_members.is_available());
}

#[test]
fn dependency_builder() {
    let dep = DependencyBuilder::default()
        .name("serde")
        .source(None)
        .req(semver::VersionReq::STAR)
        .kind(DependencyKind::Normal)
        .optional(false)
        .uses_default_features(true)
        .features(vec![])
        .target(None)
        .rename(None)
        .registry(None)
        .path(None)
        .build()
        .unwrap();

    assert_eq!(dep.name, "serde");
    assert_eq!(dep.kind, DependencyKind::Normal);
}

#[test]
fn artifact_profile_builder() {
    let profile = ArtifactProfileBuilder::default()
        .opt_level("0")
        .debuginfo(ArtifactDebuginfo::None)
        .debug_assertions(true)
        .overflow_checks(true)
        .test(false)
        .build()
        .unwrap();

    assert_eq!(profile.opt_level, "0");
    assert!(profile.debug_assertions);
}

#[test]
fn artifact_builder() {
    let pkg_id = PackageId::from("pkg 1.0.0 (path+file:///workspace)");
    let target = TargetBuilder::default()
        .name("my-lib")
        .kind(vec![TargetKind::from("lib")])
        .src_path("/workspace/src/lib.rs")
        .build()
        .unwrap();
    let profile = ArtifactProfileBuilder::default()
        .opt_level("0")
        .debuginfo(ArtifactDebuginfo::None)
        .debug_assertions(true)
        .overflow_checks(true)
        .test(false)
        .build()
        .unwrap();
    let artifact = ArtifactBuilder::default()
        .package_id(pkg_id.clone())
        .manifest_path("/workspace/Cargo.toml")
        .target(target)
        .profile(profile)
        .features(vec![])
        .filenames(vec![])
        .executable(None)
        .fresh(false)
        .build()
        .unwrap();

    assert_eq!(artifact.package_id, pkg_id);
    assert!(!artifact.fresh);
}

#[test]
fn compiler_message_builder() {
    let pkg_id = PackageId::from("pkg 1.0.0 (path+file:///workspace)");
    let target = TargetBuilder::default()
        .name("my-lib")
        .kind(vec![TargetKind::from("lib")])
        .src_path("/workspace/src/lib.rs")
        .build()
        .unwrap();
    let diagnostic = DiagnosticBuilder::default()
        .message("unused variable")
        .code(None)
        .level(DiagnosticLevel::Warning)
        .spans(vec![])
        .children(vec![])
        .rendered(None)
        .build()
        .unwrap();
    let compiler_message = CompilerMessageBuilder::default()
        .package_id(pkg_id.clone())
        .target(target)
        .message(diagnostic)
        .build()
        .unwrap();

    assert_eq!(compiler_message.package_id, pkg_id);
}

#[test]
fn build_script_builder() {
    let pkg_id = PackageId::from("pkg 1.0.0 (path+file:///workspace)");
    let build_script = BuildScriptBuilder::default()
        .package_id(pkg_id.clone())
        .linked_libs(vec![])
        .linked_paths(vec![])
        .cfgs(vec![])
        .env(vec![])
        .out_dir("/workspace/target/build/pkg/out")
        .build()
        .unwrap();

    assert_eq!(build_script.package_id, pkg_id);
}

#[test]
fn build_finished_builder() {
    let build_finished = BuildFinishedBuilder::default()
        .success(true)
        .build()
        .unwrap();

    assert!(build_finished.success);
}

#[test]
fn diagnostic_code_builder() {
    let diag_code = DiagnosticCodeBuilder::default()
        .code("E0308")
        .explanation(None)
        .build()
        .unwrap();

    assert_eq!(diag_code.code, "E0308");
}

#[test]
fn diagnostic_span_line_builder() {
    let span_line = DiagnosticSpanLineBuilder::default()
        .text("    let x = 5;")
        .highlight_start(9_usize)
        .highlight_end(14_usize)
        .build()
        .unwrap();

    assert_eq!(span_line.highlight_start, 9);
    assert_eq!(span_line.highlight_end, 14);
}

#[test]
fn diagnostic_span_builder() {
    let span_line = DiagnosticSpanLineBuilder::default()
        .text("    let x = 5;")
        .highlight_start(9_usize)
        .highlight_end(14_usize)
        .build()
        .unwrap();
    let diag_span = DiagnosticSpanBuilder::default()
        .file_name("src/main.rs")
        .byte_start(0_u32)
        .byte_end(10_u32)
        .line_start(1_usize)
        .line_end(1_usize)
        .column_start(1_usize)
        .column_end(10_usize)
        .is_primary(true)
        .text(vec![span_line])
        .label(None)
        .suggested_replacement(None)
        .suggestion_applicability(None)
        .expansion(None)
        .build()
        .unwrap();

    assert_eq!(diag_span.file_name, "src/main.rs");
    assert!(diag_span.is_primary);
}

#[test]
fn diagnostic_span_macro_expansion_builder() {
    let diag_span = DiagnosticSpanBuilder::default()
        .file_name("src/main.rs")
        .byte_start(0_u32)
        .byte_end(10_u32)
        .line_start(1_usize)
        .line_end(1_usize)
        .column_start(1_usize)
        .column_end(10_usize)
        .is_primary(false)
        .text(vec![])
        .label(None)
        .suggested_replacement(None)
        .suggestion_applicability(None)
        .expansion(None)
        .build()
        .unwrap();
    let macro_expansion = DiagnosticSpanMacroExpansionBuilder::default()
        .span(diag_span)
        .macro_decl_name("println!")
        .def_site_span(None)
        .build()
        .unwrap();

    assert_eq!(macro_expansion.macro_decl_name, "println!");
}

#[test]
fn diagnostic_builder() {
    let diagnostic = DiagnosticBuilder::default()
        .message("unused variable")
        .code(None)
        .level(DiagnosticLevel::Warning)
        .spans(vec![])
        .children(vec![])
        .rendered(None)
        .build()
        .unwrap();

    assert_eq!(diagnostic.message, "unused variable");
    assert_eq!(diagnostic.level, DiagnosticLevel::Warning);
}
