// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub mod metadata;

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use version::Version;

use std::{
    collections::{HashMap, HashSet},
    io::ErrorKind,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::{
    bash::{expr::BashExpr, vars::BashVars},
    config::bundle::{ConfigBundle, IsPackageAcceptedResult},
    data::{IUseMap, Slot, UseMap},
    dependency::{
        package::{AsPackageRef, PackageRef},
        requse::RequiredUseDependency,
        ThreeValuedPredicate,
    },
};

use self::metadata::{CachedEBuildEvaluator, EBuildBasicData, EBuildMetadata, MaybeEBuildMetadata};

/// Parses IUSE defined by ebuild/eclasses and returns as an [IUseMap].
fn parse_iuse_map(vars: &BashVars) -> Result<IUseMap> {
    Ok(vars
        .get_scalar_or_default("IUSE")?
        .split_ascii_whitespace()
        .map(|token| {
            if let Some(name) = token.strip_prefix('+') {
                return (name, true);
            }
            if let Some(name) = token.strip_prefix('-') {
                return (name, false);
            }
            (token, false)
        })
        .map(|(name, value)| (name.to_owned(), value))
        .collect())
}

/// Represents a package's readiness for installation.
#[derive(Debug, Eq, PartialEq)]
pub enum PackageReadiness {
    /// The package can be installed.
    Ok,
    /// The package is masked and cannot be installed.
    Masked { reason: String },
}

impl PackageReadiness {
    pub fn ok(&self) -> bool {
        matches!(self, PackageReadiness::Ok)
    }

    pub fn masked(&self) -> bool {
        matches!(self, PackageReadiness::Masked { .. })
    }
}

struct BashExprVisitor;

impl<'de> serde::de::Visitor<'de> for BashExprVisitor {
    type Value = BashExpr;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("simple bash expression")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        BashExpr::from_str(if value { "true" } else { "false" }).map_err(serde::de::Error::custom)
    }

    fn visit_str<E>(self, value: &str) -> std::prelude::v1::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        BashExpr::from_str(value).map_err(serde::de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for BashExpr {
    fn deserialize<D>(deserializer: D) -> Result<BashExpr, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(BashExprVisitor)
    }
}

/// Defines the merged Bazel-specific metadata found in all relevant TOML files.
///
/// Metadata of a package may consist of multiple TOML files: one for the ebuild file and those for
/// the classes inherited by the ebuild.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
pub struct BazelSpecificMetadata {
    /// Bazel target labels providing extra source code needed to build the package.
    ///
    /// The order of labels does not matter. Duplicated labels are allowed.
    /// When multiple TOML files set this metadata for the same package, labels are simply merged.
    ///
    /// # Background
    ///
    /// First-party ebuilds should usually define `CROS_WORKON_*` variables and inherit
    /// `cros-workon.eclass` to declare source code dependencies. However, it's sometimes the case
    /// that a package build needs to depend on extra source code that are not declared in the
    /// ebuild's `CROS_WORKON_*`, e.g. some common build scripts. This is especially the case with
    /// eclasses because manipulating `CROS_WORKON_*` correctly in eclasses is not straightforward.
    /// Under the Portage-orchestrated build system, accessing those extra files is as easy as just
    /// hard-coding `/mnt/host/source/...`, but it's an error under the Bazel-orchestrated build
    /// system where source code dependencies are strictly managed.
    ///
    /// This metadata allows ebuilds and eclasses to define extra source code dependencies. Each
    /// element must be a label of a Bazel target defined with `extra_sources` rule from
    /// `//bazel/portage/build_defs:extra_sources.bzl`. The rule defines a set of files to be used
    /// as extra sources.
    pub extra_sources: HashSet<String>,

    /// The package supports dynamically linking against interface only shared objects.
    ///
    /// Enabling this will result in all build-time dependencies of the package having their
    /// shared objects (.so) stripped of all code. All static libraries (.a) and executables
    /// (/bin, /usr/bin, etc) will also be omitted. By pruning the dependencies, the package will
    /// not have to rebuild unless the interface of the dependencies change.
    ///
    /// You must set this to `false` if your package performs any kind of static linking,
    /// otherwise the required files won't be present.
    ///
    /// Format: You can specify either `true`, `false`, or a shell expression. The shell
    /// expression is used to test USE flags. i.e., `use static` or `use !foo && use bar`.
    ///
    /// This value can also be declared on an `eclass` and it will propagate to all packages that
    /// inherit from it. If multiple declarations are found they are all ANDed together.
    supports_interface_libraries: Vec<BashExpr>,

    /// The static libraries that we allow into the interface library layers.
    ///
    /// You may want to do this if your package generates static libraries that
    /// always need to be consumed when linking against this package. Ideally
    /// everything would be dynamically linked, but some packages are hybrids.
    ///
    /// The path is relative to the sysroot.
    pub interface_library_allowlist: HashSet<PathBuf>,
}

impl BazelSpecificMetadata {
    /// Evaluates the `supports_interface_libraries` expressions.
    pub fn eval_supports_interface_libraries(&self, use_map: &UseMap) -> Result<bool> {
        for expr in &self.supports_interface_libraries {
            if !expr.eval(use_map).with_context(|| {
                format!("Failed evaluating {:?} with use map: {:?}", expr, use_map)
            })? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Defines the Bazel table found in a single TOML file.
///
/// This is the actual format that users will specify in the TOML file. This
/// will then be merged into the [`BazelSpecificMetadata`] which is capable of
/// holding the merged results.
#[derive(Clone, Debug, Default, Eq, Deserialize, PartialEq)]
struct SingleBazelSpecificMetadata {
    extra_sources: Option<Vec<String>>,
    supports_interface_libraries: Option<BashExpr>,
    interface_library_allowlist: Option<Vec<PathBuf>>,
}

/// Defines the TOML metadata file format.
#[derive(Clone, Debug, Default, Eq, Deserialize, PartialEq)]
struct TomlMetadata {
    bazel: Option<SingleBazelSpecificMetadata>,
}

impl BazelSpecificMetadata {
    pub fn load(ebuild_basic_data: &EBuildBasicData, eclass_paths: &[&Path]) -> Result<Self> {
        // Compute config paths.
        let ebuild_config_path = ebuild_basic_data
            .ebuild_path
            .parent()
            .expect("non-empty ebuild file path")
            .join(format!("{}.toml", ebuild_basic_data.short_package_name));
        let eclass_config_paths = eclass_paths
            .iter()
            .map(|eclass_path| eclass_path.with_extension("toml"));
        let config_paths = eclass_config_paths.chain(std::iter::once(ebuild_config_path));

        // Load configs.
        let mut merged_metadata: BazelSpecificMetadata = Default::default();
        for config_path in config_paths {
            let toml_content = match std::fs::read_to_string(&config_path) {
                Ok(toml_content) => toml_content,
                Err(e) if e.kind() == ErrorKind::NotFound => continue,
                Err(e) => {
                    return Err(e).context(format!("Failed to read {}", config_path.display()))
                }
            };

            let metadata: TomlMetadata = toml::from_str(&toml_content)
                .with_context(|| format!("Failed to parse {}", config_path.display()))?;
            merged_metadata.merge(metadata);
        }

        Ok(merged_metadata)
    }

    fn merge(&mut self, other: TomlMetadata) {
        if let Some(other) = other.bazel {
            if let Some(extra_sources) = other.extra_sources {
                self.extra_sources.extend(extra_sources);
            }
            self.supports_interface_libraries
                .extend(other.supports_interface_libraries);
            if let Some(interface_library_allowlist) = other.interface_library_allowlist {
                self.interface_library_allowlist
                    .extend(interface_library_allowlist);
            }
        }
    }
}

#[derive(Debug)]
pub struct PackageDetails {
    pub metadata: Arc<EBuildMetadata>,
    pub slot: Slot,
    pub use_map: UseMap,
    pub stable: bool,
    pub readiness: PackageReadiness,
    pub inherited: HashSet<String>,
    pub inherit_paths: Vec<PathBuf>,
    pub direct_build_target: Option<String>,
    pub bazel_metadata: BazelSpecificMetadata,
}

impl PackageDetails {
    /// EAPI is technically a string, but working with an integer is easier.
    fn eapi(&self) -> Result<i32> {
        let eapi = self.metadata.vars.get_scalar("EAPI")?;
        eapi.parse::<i32>().with_context(|| format!("EAPI: {eapi}"))
    }

    pub fn supports_bdepend(&self) -> bool {
        let eapi = match self.eapi() {
            Ok(val) => val,
            Err(_) => return false,
        };

        eapi >= 7
    }

    pub fn supports_idepend(&self) -> bool {
        let eapi = match self.eapi() {
            Ok(val) => val,
            Err(_) => return false,
        };

        eapi >= 8
    }
}

impl PackageDetails {
    pub fn as_basic_data(&self) -> &EBuildBasicData {
        &self.metadata.basic_data
    }

    pub fn as_metadata(&self) -> &EBuildMetadata {
        &self.metadata
    }
}

impl AsPackageRef for PackageDetails {
    fn as_package_ref(&self) -> PackageRef {
        PackageRef {
            package_name: &self.as_basic_data().package_name,
            version: &self.as_basic_data().version,
            slot: Some(Slot {
                main: self.slot.main.as_str(),
                sub: self.slot.sub.as_str(),
            }),
            use_map: Some(&self.use_map),
            readiness: Some(self.readiness.ok()),
        }
    }
}

/// Represents an error that occurred when loading an ebuild.
#[derive(Debug)]
pub struct PackageLoadError {
    pub metadata: MaybeEBuildMetadata,
    pub error: String,
}

impl PackageLoadError {
    pub fn as_basic_data(&self) -> &EBuildBasicData {
        self.metadata.as_basic_data()
    }
}

impl AsPackageRef for PackageLoadError {
    fn as_package_ref(&self) -> PackageRef {
        self.metadata.as_package_ref()
    }
}

/// Represents a package, covering both successfully loaded ones and failed ones.
///
/// Since this enum is very lightweight (contains [`Arc`] only), you should not wrap it within
/// reference-counting smart pointers like [`Arc`], but you can just clone it.
///
/// While this enum looks very similar to [`Result`], we don't make it a type alias of [`Result`]
/// to implement a few convenient methods.
#[derive(Clone, Debug)]
pub enum MaybePackageDetails {
    Ok(Arc<PackageDetails>),
    Err(Arc<PackageLoadError>),
}

impl MaybePackageDetails {
    pub fn as_basic_data(&self) -> &EBuildBasicData {
        match self {
            MaybePackageDetails::Ok(details) => details.as_basic_data(),
            MaybePackageDetails::Err(error) => error.as_basic_data(),
        }
    }
}

impl AsPackageRef for MaybePackageDetails {
    fn as_package_ref(&self) -> PackageRef {
        match self {
            MaybePackageDetails::Ok(details) => details.as_package_ref(),
            MaybePackageDetails::Err(error) => error.as_package_ref(),
        }
    }
}

#[derive(Debug)]
pub struct PackageLoader {
    evaluator: Arc<CachedEBuildEvaluator>,
    config: Arc<ConfigBundle>,
    force_accept_9999_ebuilds: bool,
    version_9999: Version,
}

impl PackageLoader {
    pub fn new(
        evaluator: Arc<CachedEBuildEvaluator>,
        config: Arc<ConfigBundle>,
        force_accept_9999_ebuilds: bool,
    ) -> Self {
        Self {
            evaluator,
            config,
            force_accept_9999_ebuilds,
            version_9999: Version::try_new("9999").unwrap(),
        }
    }

    /// Loads a package information from a specified ebuild path.
    pub fn load_package(&self, ebuild_path: &Path) -> Result<MaybePackageDetails> {
        let metadata = self.evaluator.evaluate_metadata(ebuild_path)?;

        // Don't abort on package parse failures.
        match self.parse_package(metadata.clone()) {
            Ok(details) => Ok(MaybePackageDetails::Ok(Arc::new(details))),
            Err(error) => Ok(MaybePackageDetails::Err(Arc::new(PackageLoadError {
                metadata,
                error: error.to_string(),
            }))),
        }
    }

    /// Parses [`MaybeEBuildMetadata`] into [`PackageDetails`].
    fn parse_package(&self, metadata: MaybeEBuildMetadata) -> Result<PackageDetails> {
        let package_name = format!(
            "{}/{}",
            metadata.as_basic_data().category_name,
            metadata.as_basic_data().short_package_name
        );

        let metadata = match metadata {
            MaybeEBuildMetadata::Ok(metadata) => metadata,
            MaybeEBuildMetadata::Err(error) => {
                bail!("{}", error.error);
            }
        };

        let slot = Slot::<String>::new(metadata.vars.get_scalar("SLOT")?);

        let package = PackageRef {
            package_name: package_name.as_str(),
            version: &metadata.basic_data.version,
            slot: Some(Slot {
                main: &slot.main,
                sub: &slot.sub,
            }),
            use_map: None,
            readiness: None,
        };

        let raw_inherited = metadata.vars.get_scalar_or_default("INHERITED")?;
        let inherited: HashSet<String> = raw_inherited
            .split_ascii_whitespace()
            .map(|s| s.to_owned())
            .collect();

        let raw_inherit_paths = metadata
            .vars
            .get_indexed_array("__alchemist_out_inherit_paths")?;
        let inherit_paths: Vec<PathBuf> = raw_inherit_paths.iter().map(PathBuf::from).collect();

        let accepted_result = self.config.is_package_accepted(&metadata.vars, &package)?;
        let accepted_result = (|| {
            if matches!(&accepted_result, IsPackageAcceptedResult::Unaccepted { .. })
                && self.force_accept_9999_ebuilds
            {
                let auto_uprev = match metadata.vars.get_scalar("CROS_WORKON_MANUAL_UPREV") {
                    Ok(value) => value != "1",
                    Err(_) => false,
                };
                if inherited.contains("cros-workon")
                    && metadata.basic_data.version == self.version_9999
                    && auto_uprev
                {
                    return IsPackageAcceptedResult::Accepted { stable: false };
                }
            }
            accepted_result
        })();

        let stable = match &accepted_result {
            IsPackageAcceptedResult::Unaccepted { .. } => false,
            IsPackageAcceptedResult::Accepted { stable } => *stable,
        };

        let iuse_map = parse_iuse_map(&metadata.vars)?;
        let use_map = self.config.compute_use_map(
            &package_name,
            &metadata.basic_data.version,
            stable,
            &slot,
            &iuse_map,
        );

        let raw_required_use = metadata.vars.get_scalar_or_default("REQUIRED_USE")?;
        let required_use: RequiredUseDependency = raw_required_use.parse()?;

        let readiness = if let IsPackageAcceptedResult::Unaccepted { reason } = accepted_result {
            PackageReadiness::Masked { reason }
        } else if self.config.is_package_masked(&package) {
            // TODO: Give a better explanation.
            PackageReadiness::Masked {
                reason: "Masked by configs".into(),
            }
        } else if required_use.matches(&use_map, &())? == Some(false) {
            PackageReadiness::Masked {
                reason: format!("REQUIRED_USE not satisfied: {}", raw_required_use),
            }
        } else {
            PackageReadiness::Ok
        };

        let direct_build_target = metadata
            .vars
            .maybe_get_scalar("METALLURGY_TARGET")?
            .map(|s| {
                if s.starts_with('@') {
                    s.to_string()
                } else {
                    // eg. //bazel:foo -> @@//bazel:foo
                    format!("@@{s}")
                }
            });

        let bazel_metadata = BazelSpecificMetadata::load(
            metadata.as_basic_data(),
            &inherit_paths.iter().map(|p| p.as_path()).collect_vec(),
        )?;

        Ok(PackageDetails {
            metadata,
            slot,
            use_map,
            stable,
            readiness,
            inherited,
            inherit_paths,
            direct_build_target,
            bazel_metadata,
        })
    }
}

/// Wraps PackageLoader to cache results.
#[derive(Debug)]
pub struct CachedPackageLoader {
    loader: PackageLoader,
    cache: Mutex<HashMap<PathBuf, Arc<OnceCell<MaybePackageDetails>>>>,
}

impl CachedPackageLoader {
    pub fn new(loader: PackageLoader) -> Self {
        Self {
            loader,
            cache: Default::default(),
        }
    }

    pub fn load_package(&self, ebuild_path: &Path) -> Result<MaybePackageDetails> {
        let once_cell = {
            let mut cache_guard = self.cache.lock().unwrap();
            cache_guard
                .entry(ebuild_path.to_owned())
                .or_default()
                .clone()
        };
        let details = once_cell.get_or_try_init(|| self.loader.load_package(ebuild_path))?;
        Ok(details.clone())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::bool_assert_comparison)]

    use std::str::FromStr;

    use tempfile::TempDir;

    use crate::repository::{RepositoryLayout, RepositorySet};

    use super::*;

    fn do_load_package(
        ebuild_relative_path: &str,
        ebuild_content: &str,
    ) -> Result<MaybePackageDetails> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.path();

        let ebuild_path = temp_dir.join(ebuild_relative_path);
        std::fs::create_dir_all(ebuild_path.parent().unwrap())?;
        std::fs::write(&ebuild_path, ebuild_content)?;

        let repo_set = RepositorySet::load_from_layouts(
            "test",
            &[RepositoryLayout::new("test", temp_dir, &[])],
        )?;

        let evaluator = CachedEBuildEvaluator::new(
            repo_set.get_repos().into_iter().cloned().collect(),
            &temp_dir.join("tools"),
        );

        let config = ConfigBundle::new_for_testing("riscv");
        let loader = PackageLoader::new(Arc::new(evaluator), Arc::new(config), false);

        loader.load_package(&ebuild_path)
    }

    fn do_load_package_and_unwrap(
        ebuild_relative_path: &str,
        ebuild_content: &str,
    ) -> Arc<PackageDetails> {
        let maybe_details = do_load_package(ebuild_relative_path, ebuild_content).unwrap();

        match maybe_details {
            MaybePackageDetails::Ok(details) => details,
            MaybePackageDetails::Err(error) => panic!("Failed to load package: {error:?}"),
        }
    }

    #[test]
    fn test_load_success() {
        let details = do_load_package_and_unwrap(
            "sys-apps/hello/hello-1.ebuild",
            r#"
EAPI=7
SLOT=0
KEYWORDS="*"
"#,
        );

        // Verify `PackageDetails` fields, except `metadata` that is tested by
        // unit tests in `metadata.rs`.
        assert_eq!(details.slot, Slot::new("0"));
        assert_eq!(details.use_map, UseMap::new());
        assert_eq!(details.stable, true);
        assert_eq!(details.readiness, PackageReadiness::Ok);
        assert_eq!(details.inherited, HashSet::new());
        assert_eq!(details.inherit_paths, Vec::<PathBuf>::new());
        assert_eq!(details.direct_build_target, None);
    }

    #[test]
    fn test_load_iuse() {
        let details = do_load_package_and_unwrap(
            "sys-apps/hello/hello-1.ebuild",
            r#"
EAPI=7
SLOT=0
KEYWORDS="*"
IUSE="foo +bar"
"#,
        );
        assert_eq!(
            details.use_map,
            UseMap::from_iter([("foo".into(), false), ("bar".into(), true)])
        );
    }

    #[test]
    fn test_load_keywords() {
        let details = do_load_package_and_unwrap(
            "sys-apps/hello/hello-1.ebuild",
            r#"
EAPI=7
SLOT=0
KEYWORDS="-*"
"#,
        );
        assert_eq!(details.stable, false);
        assert_eq!(
            details.readiness,
            PackageReadiness::Masked {
                reason: "KEYWORDS (-*) is not accepted by ACCEPT_KEYWORDS (riscv)".into()
            }
        );
    }

    #[test]
    fn test_load_inherit_paths() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.path();

        // Create an ebuild.
        let ebuild_dir = temp_dir.join("sys-apps/hello");
        let ebuild_path = ebuild_dir.join("hello-1.2.3.ebuild");
        std::fs::create_dir_all(&ebuild_dir)?;
        std::fs::write(
            &ebuild_path,
            r#"
            EAPI=7
            SLOT=0
            KEYWORDS="*"
            inherit aaa
        "#,
        )?;

        // Create eclasses with a diamond inheritance.
        let eclass_dir = temp_dir.join("eclass");
        std::fs::create_dir_all(&eclass_dir)?;
        std::fs::write(eclass_dir.join("aaa.eclass"), "inherit bbb ccc")?;
        std::fs::write(eclass_dir.join("bbb.eclass"), "inherit ddd")?;
        std::fs::write(eclass_dir.join("ccc.eclass"), "inherit ddd")?;
        std::fs::write(eclass_dir.join("ddd.eclass"), "")?;

        // Load the package.
        let repo_set = RepositorySet::load_from_layouts(
            "test",
            &[RepositoryLayout::new("test", temp_dir, &[])],
        )?;

        let evaluator = CachedEBuildEvaluator::new(
            repo_set.get_repos().into_iter().cloned().collect(),
            &temp_dir.join("tools"),
        );

        let config = ConfigBundle::new_for_testing("riscv");
        let loader = PackageLoader::new(Arc::new(evaluator), Arc::new(config), false);

        let maybe_details = loader.load_package(&ebuild_path)?;

        let details = match maybe_details {
            MaybePackageDetails::Ok(details) => details,
            MaybePackageDetails::Err(error) => bail!("Failed to load package: {error:?}"),
        };

        // Verify the inherit paths.
        assert_eq!(
            details.inherit_paths,
            vec![
                eclass_dir.join("ddd.eclass"),
                eclass_dir.join("bbb.eclass"),
                eclass_dir.join("ddd.eclass"),
                eclass_dir.join("ccc.eclass"),
                eclass_dir.join("aaa.eclass"),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_load_required_use() {
        let details = do_load_package_and_unwrap(
            "sys-apps/hello/hello-1.ebuild",
            r#"
EAPI=7
SLOT=0
KEYWORDS="*"
IUSE="foo +bar"
REQUIRED_USE="|| ( foo !bar )"
"#,
        );
        assert_eq!(details.stable, true);
        assert_eq!(
            details.readiness,
            PackageReadiness::Masked {
                reason: "REQUIRED_USE not satisfied: || ( foo !bar )".into()
            }
        );
    }

    #[test]
    fn test_load_bazel_metadata() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.path();

        // Create an ebuild and its associated toml.
        let ebuild_dir = temp_dir.join("sys-apps/hello");
        let ebuild_path = ebuild_dir.join("hello-1.2.3.ebuild");
        std::fs::create_dir_all(&ebuild_dir)?;
        std::fs::write(
            &ebuild_path,
            r#"
            EAPI=7
            SLOT=0
            KEYWORDS="*"
            inherit foo
        "#,
        )?;
        std::fs::write(
            ebuild_dir.join("hello.toml"),
            r#"
            [bazel]
            extra_sources = [
                "//platform2/common-mk:sources",
                "//scripts:sources",
            ]
            supports_interface_libraries = true
        "#,
        )?;

        // Create eclasses and their associated toml.
        let eclass_dir = temp_dir.join("eclass");
        std::fs::create_dir_all(&eclass_dir)?;
        std::fs::write(eclass_dir.join("foo.eclass"), "inherit bar")?;
        // foo.toml is missing.
        std::fs::write(eclass_dir.join("bar.eclass"), "")?;
        std::fs::write(
            eclass_dir.join("bar.toml"),
            r#"
            [bazel]
            extra_sources = [
                "@chromite//:sources",
                "//scripts:sources",
            ]
        "#,
        )?;

        // Load the package.
        let repo_set = RepositorySet::load_from_layouts(
            "test",
            &[RepositoryLayout::new("test", temp_dir, &[])],
        )?;

        let evaluator = CachedEBuildEvaluator::new(
            repo_set.get_repos().into_iter().cloned().collect(),
            &temp_dir.join("tools"),
        );

        let config = ConfigBundle::new_for_testing("riscv");
        let loader = PackageLoader::new(Arc::new(evaluator), Arc::new(config), false);

        let maybe_details = loader.load_package(&ebuild_path)?;

        let details = match maybe_details {
            MaybePackageDetails::Ok(details) => details,
            MaybePackageDetails::Err(error) => bail!("Failed to load package: {error:?}"),
        };

        // Verify the Bazel-specific metadata.
        assert_eq!(
            details.bazel_metadata,
            BazelSpecificMetadata {
                extra_sources: HashSet::from([
                    "//platform2/common-mk:sources".into(),
                    "//scripts:sources".into(),
                    "@chromite//:sources".into(),
                ]),
                supports_interface_libraries: vec![BashExpr::from_str("true")?],
                interface_library_allowlist: HashSet::from([]),
            }
        );
        Ok(())
    }

    #[test]
    fn test_load_fatal_error() {
        let result = do_load_package(
            // Invalid file name.
            "sys-apps/hello/hello-1.eclass",
            r#"
EAPI=7
SLOT=0
KEYWORDS="*"
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_load_parse_error() {
        let maybe_details = do_load_package(
            "sys-apps/hello/hello-1.ebuild",
            r#"
EAPI=7
SLOT=("0" "0")  # SLOT is an array!
KEYWORDS="*"
"#,
        )
        .expect("load_package should return success despite the parse error");
        matches!(maybe_details, MaybePackageDetails::Err(_));
    }

    fn write_toml(
        package_toml: &str,
        eclass_toml: &[(&str, &str)],
    ) -> Result<BazelSpecificMetadata> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.path();

        let ebuild_dir = temp_dir.join("sys-apps/hello");
        std::fs::create_dir_all(&ebuild_dir)?;

        let ebuild_path = ebuild_dir.join("hello-1.0.ebuild");
        std::fs::write(ebuild_dir.join("hello.toml"), package_toml)?;

        let eclass_dir = temp_dir.join("eclass");
        std::fs::create_dir_all(&eclass_dir)?;
        let mut eclass_paths = vec![];
        for (key, value) in eclass_toml {
            std::fs::write(eclass_dir.join(format!("{key}.toml")), value)?;

            let eclass_path = eclass_dir.join(format!("{key}.eclass"));
            eclass_paths.push(eclass_path);
        }

        BazelSpecificMetadata::load(
            &EBuildBasicData {
                repo_name: "repo".to_string(),
                ebuild_path,
                package_name: "sys-apps/hello".into(),
                short_package_name: "hello".into(),
                category_name: "sys-apps".into(),
                version: Version::from_str("1.0")?,
            },
            &eclass_paths.iter().map(|p| p.as_path()).collect_vec(),
        )
    }

    #[test]
    fn test_empty_toml_parsing() -> Result<()> {
        let metadata = BazelSpecificMetadata {
            extra_sources: HashSet::from([]),
            supports_interface_libraries: vec![],
            interface_library_allowlist: HashSet::from([]),
        };

        assert_eq!(write_toml("", &[])?, metadata);

        assert!(metadata.eval_supports_interface_libraries(&HashMap::from([]))?);
        Ok(())
    }

    #[test]
    fn test_bool_toml_parsing() -> Result<()> {
        let metadata = BazelSpecificMetadata {
            extra_sources: HashSet::from([]),
            supports_interface_libraries: vec![BashExpr::from_str("false")?],
            interface_library_allowlist: HashSet::from([]),
        };

        assert_eq!(
            write_toml(
                r#"
[bazel]
supports_interface_libraries = false
                "#,
                &[]
            )?,
            metadata
        );

        assert!(!metadata.eval_supports_interface_libraries(&HashMap::from([]))?);

        let metadata = BazelSpecificMetadata {
            extra_sources: HashSet::from([]),
            supports_interface_libraries: vec![BashExpr::from_str("true")?],
            interface_library_allowlist: HashSet::from([]),
        };

        assert_eq!(
            write_toml(
                r#"
[bazel]
supports_interface_libraries = true
                "#,
                &[]
            )?,
            metadata
        );

        assert!(metadata.eval_supports_interface_libraries(&HashMap::from([]))?);

        Ok(())
    }

    #[test]
    fn test_str_toml_parsing() -> Result<()> {
        let metadata = BazelSpecificMetadata {
            extra_sources: HashSet::from([]),
            supports_interface_libraries: vec![BashExpr::from_str("use !static")?],
            interface_library_allowlist: HashSet::from([]),
        };

        assert_eq!(
            write_toml(
                r#"
[bazel]
supports_interface_libraries = "use !static"
                "#,
                &[]
            )?,
            metadata
        );

        assert!(!metadata
            .eval_supports_interface_libraries(&HashMap::from([("static".into(), true)]))?);

        assert!(metadata
            .eval_supports_interface_libraries(&HashMap::from([("static".into(), false)]))?);

        Ok(())
    }

    #[test]
    fn test_toml_overrides() -> Result<()> {
        let metadata = BazelSpecificMetadata {
            extra_sources: HashSet::from([]),
            supports_interface_libraries: vec![BashExpr::from_str("true")?],
            interface_library_allowlist: HashSet::from([]),
        };

        assert_eq!(
            write_toml(
                "",
                &[(
                    "foo",
                    r#"
[bazel]
supports_interface_libraries = true
"#
                )]
            )?,
            metadata
        );

        assert!(metadata.eval_supports_interface_libraries(&HashMap::from([]))?);

        let metadata = BazelSpecificMetadata {
            extra_sources: HashSet::from([]),
            supports_interface_libraries: vec![
                BashExpr::from_str("true")?,
                BashExpr::from_str("false")?,
            ],
            interface_library_allowlist: HashSet::from([]),
        };

        // Verify packages can override the eclasses.
        assert_eq!(
            write_toml(
                r#"
                [bazel]
supports_interface_libraries = false
                "#,
                &[(
                    "foo",
                    r#"
[bazel]
supports_interface_libraries = true
"#
                )]
            )?,
            metadata
        );

        assert!(!metadata.eval_supports_interface_libraries(&HashMap::from([]))?);

        Ok(())
    }

    #[test]
    fn test_toml_interface_library_allowlist() -> Result<()> {
        let metadata = BazelSpecificMetadata {
            extra_sources: HashSet::from([]),
            supports_interface_libraries: vec![],
            interface_library_allowlist: HashSet::from([
                PathBuf::from("/usr/lib/baz.a"),
                PathBuf::from("/usr/lib/foo.a"),
                PathBuf::from("/usr/lib/bar.a"),
            ]),
        };

        assert_eq!(
            write_toml(
                r#"
[bazel]
interface_library_allowlist = [
    "/usr/lib/baz.a",
]
"#,
                &[(
                    "foo",
                    r#"
[bazel]
interface_library_allowlist = [
    "/usr/lib/foo.a",
    "/usr/lib/bar.a",
]
"#
                )]
            )?,
            metadata
        );

        Ok(())
    }
}
