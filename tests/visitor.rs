extern crate cargo_metadata;
extern crate semver;

use std::collections::{BTreeMap, BTreeSet};

use cargo_metadata::{
    CargoOpt, FeatureValue, FeatureVisitor, FeatureWalker, MetadataCommand, Package, PackageId,
};

#[derive(Default, Debug)]
struct TransitiveFeatureCollector {
    features: BTreeMap<PackageId, BTreeSet<FeatureValue>>,
    err_on_missing: bool,
}

impl TransitiveFeatureCollector {
    fn new(err_on_missing: bool) -> Self {
        Self {
            features: BTreeMap::default(),
            err_on_missing,
        }
    }

    fn collect_feature_value(&mut self, package_id: PackageId, feature_value: FeatureValue) {
        self.features
            .entry(package_id)
            .or_default()
            .insert(feature_value);
    }
}

impl FeatureVisitor for TransitiveFeatureCollector {
    type Error = String;

    fn visit_missing_dependency(&mut self, dep_name: &str) -> Result<(), Self::Error> {
        if self.err_on_missing {
            Err(format!("missing dependency: {dep_name:?}"))
        } else {
            Ok(())
        }
    }

    fn visit_missing_package(&mut self, pkg_name: &str) -> Result<(), Self::Error> {
        if self.err_on_missing {
            Err(format!("missing package: {pkg_name:?}"))
        } else {
            Ok(())
        }
    }

    fn visit_feature(
        &mut self,
        package: &Package,
        feature_name: &str,
    ) -> Result<bool, Self::Error> {
        self.collect_feature_value(
            package.id.clone(),
            FeatureValue::Feature(feature_name.to_owned()),
        );

        Ok(true)
    }

    fn visit_dep(&mut self, package: &Package, dep_name: &str) -> Result<bool, Self::Error> {
        self.collect_feature_value(
            package.id.clone(),
            FeatureValue::Dep {
                dep_name: dep_name.to_owned(),
            },
        );

        Ok(true)
    }

    fn visit_dep_feature(
        &mut self,
        package: &Package,
        dep_name: &str,
        dep_feature: &str,
        weak: bool,
    ) -> Result<bool, Self::Error> {
        self.collect_feature_value(
            package.id.clone(),
            FeatureValue::DepFeature {
                dep_name: dep_name.to_owned(),
                dep_feature: dep_feature.to_owned(),
                weak: weak,
            },
        );

        Ok(true)
    }
}

#[test]
fn all_features() -> Result<(), cargo_metadata::Error> {
    let mut cmd = MetadataCommand::new();
    cmd.manifest_path("tests/all/Cargo.toml");
    let meta = cmd.exec().unwrap();

    let root_package = meta.root_package().unwrap();

    let walker = FeatureWalker::new(&meta);

    let mut collector = TransitiveFeatureCollector::new(false);

    let mut features = match walker.walk_package_features(
        root_package,
        root_package.features.keys(),
        &mut collector,
    ) {
        Ok(()) => collector.features,
        Err(err) => panic!("{err}"),
    };
    assert_eq!(features.len(), 1);

    let package_features = features.remove(&root_package.id).unwrap();
    assert_eq!(
        package_features,
        BTreeSet::from_iter([
            FeatureValue::Feature("bitflags".to_owned(),),
            FeatureValue::Feature("default".to_owned(),),
            FeatureValue::Feature("feat1".to_owned(),),
            FeatureValue::Feature("feat2".to_owned(),),
            FeatureValue::Feature("opt-feat-strong".to_owned(),),
            FeatureValue::Feature("opt-feat-weak".to_owned(),),
            FeatureValue::Feature("optdep".to_owned(),),
            FeatureValue::Dep {
                dep_name: "bitflags".to_owned(),
            },
            FeatureValue::Dep {
                dep_name: "optdep".to_owned(),
            },
            FeatureValue::DepFeature {
                dep_name: "optdep".to_owned(),
                dep_feature: "feat".to_owned(),
                weak: false,
            },
            FeatureValue::DepFeature {
                dep_name: "optdep".to_owned(),
                dep_feature: "feat".to_owned(),
                weak: true,
            },
        ])
    );

    Ok(())
}

#[test]
fn default_features() -> Result<(), cargo_metadata::Error> {
    let mut cmd = MetadataCommand::new();
    cmd.manifest_path("tests/all/Cargo.toml");
    let meta = cmd.exec().unwrap();

    let root_package = meta.root_package().unwrap();

    let walker = FeatureWalker::new(&meta);

    let mut collector = TransitiveFeatureCollector::new(false);

    let mut features = match walker.walk_package_features(root_package, ["default"], &mut collector)
    {
        Ok(()) => collector.features,
        Err(err) => panic!("{err}"),
    };
    assert_eq!(features.len(), 1);

    let package_features = features.remove(&root_package.id).unwrap();
    assert_eq!(
        package_features,
        BTreeSet::from_iter([
            FeatureValue::Feature("bitflags".to_owned(),),
            FeatureValue::Feature("default".to_owned(),),
            FeatureValue::Feature("feat1".to_owned(),),
            FeatureValue::Dep {
                dep_name: "bitflags".to_owned(),
            },
        ])
    );

    Ok(())
}

#[test]
fn strong_dep_feature() -> Result<(), cargo_metadata::Error> {
    let mut cmd = MetadataCommand::new();
    cmd.manifest_path("tests/all/Cargo.toml");
    // Without this the `optdep` package will be missing on `Metadata`:
    cmd.features(CargoOpt::SomeFeatures(vec!["opt-feat-strong".to_owned()]));
    let meta = cmd.exec().unwrap();

    let root_package = meta.root_package().unwrap();

    let walker = FeatureWalker::new(&meta);

    let mut collector = TransitiveFeatureCollector::new(false);

    let mut features =
        match walker.walk_package_features(root_package, ["opt-feat-strong"], &mut collector) {
            Ok(()) => collector.features,
            Err(err) => panic!("{err}"),
        };
    assert_eq!(features.len(), 2);

    let package_features = features.remove(&root_package.id).unwrap();
    assert_eq!(
        package_features,
        BTreeSet::from_iter([
            FeatureValue::Feature("opt-feat-strong".to_owned(),),
            FeatureValue::DepFeature {
                dep_name: "optdep".to_owned(),
                dep_feature: "feat".to_owned(),
                weak: false,
            },
        ])
    );

    Ok(())
}

#[test]
fn weak_dep_feature() -> Result<(), cargo_metadata::Error> {
    let mut cmd = MetadataCommand::new();
    cmd.manifest_path("tests/all/Cargo.toml");
    // Without this the `optdep` package will be missing on `Metadata`:
    cmd.features(CargoOpt::SomeFeatures(vec!["opt-feat-strong".to_owned()]));
    let meta = cmd.exec().unwrap();

    let root_package = meta.root_package().unwrap();

    let walker = FeatureWalker::new(&meta);

    let mut collector = TransitiveFeatureCollector::new(false);

    let mut features =
        match walker.walk_package_features(root_package, ["opt-feat-weak"], &mut collector) {
            Ok(()) => collector.features,
            Err(err) => panic!("{err}"),
        };
    assert_eq!(features.len(), 2);

    let package_features = features.remove(&root_package.id).unwrap();
    assert_eq!(
        package_features,
        BTreeSet::from_iter([
            FeatureValue::Feature("opt-feat-weak".to_owned(),),
            FeatureValue::DepFeature {
                dep_name: "optdep".to_owned(),
                dep_feature: "feat".to_owned(),
                weak: true,
            },
        ])
    );

    Ok(())
}
