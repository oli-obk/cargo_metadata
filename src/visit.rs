use std::collections::BTreeMap;

use crate::{FeatureValue, Metadata, Package};

/// A visitor over a package's features and their dependencies.
pub trait FeatureVisitor {
    /// The error type of a walk.
    type Error;

    /// Visits a missing dependency.
    ///
    /// This error should not happen for valid manifests,
    /// but can happen when reading `Metadata` from unchecked JSON.
    ///
    /// Return `Ok(())` to continue the walk, or `Err(…)` to abort it.
    fn visit_missing_dependency(&mut self, dep_name: &str) -> Result<(), Self::Error>;

    /// Visits a missing package.
    ///
    /// This is usually caused by the package being an optional dependency and
    /// not having been enabled by the features that were passed to `MetadataCommand`,
    /// but it can also happen when reading `Metadata` from unchecked JSON.
    ///
    /// Return `Ok(())` to continue the walk, or `Err(…)` to abort it.
    fn visit_missing_package(&mut self, pkg_name: &str) -> Result<(), Self::Error>;

    /// Visits a feature on `package` that's enabling another feature `feature_name`.
    ///
    /// Corresponds to features with `feature_name` syntax.
    ///
    /// Return `Ok(<walk>)` where `<walk>` indicates whether or not to walk
    /// the feature's downstream dependencies, or `Err(…)` to abort the walk.
    fn visit_feature(
        &mut self,
        package: &Package,
        feature_name: &str,
    ) -> Result<bool, Self::Error> {
        let (..) = (package, feature_name);
        Ok(true)
    }

    /// Visits a feature on `package` that's enabling its dependency `dep_name`.
    ///
    /// Corresponds to features with `dep:dep_name` syntax.
    ///
    /// Return `Ok(<walk>)` where `<walk>` indicates whether or not to walk
    /// the feature's downstream dependencies, or `Err(…)` to abort the walk.
    fn visit_dep(&mut self, package: &Package, dep_name: &str) -> Result<bool, Self::Error> {
        let (..) = (package, dep_name);
        Ok(true)
    }

    /// Visits a feature on `package` that's enabling feature `dep_feature`
    /// on its dependency `dep_name`.
    ///
    /// Corresponds to features with `dep_name/dep_feature`/`dep_name?/dep_feature` syntax.
    ///
    /// Return `Ok(<walk>)` where `<walk>` indicates whether or not to walk
    /// the feature's downstream dependencies, or `Err(…)` to abort the walk.
    fn visit_dep_feature(
        &mut self,
        package: &Package,
        dep_name: &str,
        dep_feature: &str,
        weak: bool,
    ) -> Result<bool, Self::Error> {
        let (..) = (package, dep_name, dep_feature, weak);
        Ok(true)
    }
}

/// A type for walking package features and their dependencies.
pub struct FeatureWalker<'a> {
    packages_by_name: BTreeMap<String, &'a Package>,
}

impl<'a> FeatureWalker<'a> {
    /// Creates a walker for a given `metadata`.
    pub fn new(metadata: &'a Metadata) -> Self {
        let packages_by_name = metadata
            .packages
            .iter()
            .map(|package| (package.name.clone(), package))
            .collect();
        Self { packages_by_name }
    }

    /// Walks the selected features of a package and their dependencies.
    pub fn walk_package_features<V, I, F>(
        &self,
        package: &Package,
        feature_names: I,
        visitor: &mut V,
    ) -> Result<(), V::Error>
    where
        V: FeatureVisitor,
        I: IntoIterator<Item = F>,
        F: AsRef<str>,
    {
        for feature_name in feature_names {
            self.walk_feature(package, feature_name.as_ref(), visitor)?;
        }

        Ok(())
    }

    fn walk_feature<V>(
        &self,
        package: &Package,
        feature_name: &str,
        visitor: &mut V,
    ) -> Result<(), V::Error>
    where
        V: FeatureVisitor,
    {
        let Some(required_features) = package.features.get(feature_name) else {
            return Ok(());
        };

        if !visitor.visit_feature(package, feature_name)? {
            return Ok(());
        }

        for required_feature in required_features {
            self.walk_feature_value(package, required_feature, visitor)?;
        }

        Ok(())
    }

    fn walk_dep<V>(
        &self,
        package: &Package,
        dep_name: &str,
        visitor: &mut V,
    ) -> Result<(), V::Error>
    where
        V: FeatureVisitor,
    {
        let Some(dependency) = package.get_dependency(dep_name) else {
            return visitor.visit_missing_dependency(dep_name);
        };

        if !visitor.visit_dep(package, dep_name)? {
            return Ok(());
        }

        let package_name = &dependency.name;

        match self.packages_by_name.get(package_name) {
            Some(&dep_package) => {
                for dep_feature in dependency.features.iter() {
                    let dep_feature = FeatureValue::new(dep_feature);
                    assert!(matches!(dep_feature, FeatureValue::Feature(_)));
                    self.walk_feature_value(dep_package, &dep_feature, visitor)?;
                }
            }
            None => visitor.visit_missing_package(package_name)?,
        }

        Ok(())
    }

    fn walk_dep_feature<V>(
        &self,
        package: &Package,
        dep_name: &str,
        dep_feature: &str,
        weak: bool,
        visitor: &mut V,
    ) -> Result<(), V::Error>
    where
        V: FeatureVisitor,
    {
        let Some(dependency) = package.get_dependency(dep_name) else {
            return visitor.visit_missing_dependency(dep_name);
        };

        if !visitor.visit_dep_feature(package, dep_name, dep_feature, weak)? {
            return Ok(());
        }

        let package_name = &dependency.name;

        let Some(&dep_package) = self.packages_by_name.get(package_name) else {
            return visitor.visit_missing_package(package_name);
        };

        let dep_feature = FeatureValue::new(dep_feature);
        assert!(matches!(dep_feature, FeatureValue::Feature(_)));
        self.walk_feature_value(dep_package, &dep_feature, visitor)?;

        if !weak {
            for feature_name in dependency.features.iter() {
                let dep_feature = FeatureValue::new(feature_name);
                assert!(matches!(dep_feature, FeatureValue::Feature(_)));
                self.walk_feature_value(dep_package, &dep_feature, visitor)?;
            }
        }

        Ok(())
    }

    fn walk_feature_value<V>(
        &self,
        package: &Package,
        feature_value: &FeatureValue,
        visitor: &mut V,
    ) -> Result<(), V::Error>
    where
        V: FeatureVisitor,
    {
        match feature_value {
            FeatureValue::Feature(feature_name) => {
                self.walk_feature(package, feature_name, visitor)
            }
            FeatureValue::Dep { dep_name } => self.walk_dep(package, dep_name, visitor),
            FeatureValue::DepFeature {
                dep_name,
                dep_feature,
                weak,
            } => self.walk_dep_feature(package, dep_name, dep_feature, *weak, visitor),
        }
    }
}
