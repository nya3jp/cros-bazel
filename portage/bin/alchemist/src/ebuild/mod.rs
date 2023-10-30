// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub mod metadata;

use anyhow::{bail, Context, Result};
use once_cell::sync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use version::Version;

use crate::{
    bash::vars::BashVars,
    config::bundle::{ConfigBundle, IsPackageAcceptedResult},
    data::{IUseMap, Slot, UseMap},
    dependency::{
        package::{PackageRef, ThinPackageRef},
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

#[derive(Debug)]
pub struct PackageDetails {
    pub metadata: Arc<EBuildMetadata>,
    pub slot: Slot,
    pub use_map: UseMap,
    pub accepted: bool,
    pub stable: bool,
    pub masked: bool,
    pub inherited: HashSet<String>,
    pub inherit_paths: Vec<PathBuf>,
    pub direct_build_target: Option<String>,
}

impl PackageDetails {
    /// Converts this PackageDetails to a PackageRef that can be passed to
    /// dependency predicates.
    pub fn as_package_ref(&self) -> PackageRef {
        PackageRef {
            package_name: &self.as_basic_data().package_name,
            version: &self.as_basic_data().version,
            slot: Slot {
                main: self.slot.main.as_str(),
                sub: self.slot.sub.as_str(),
            },
            use_map: &self.use_map,
        }
    }

    pub fn as_thin_package_ref(&self) -> ThinPackageRef {
        ThinPackageRef {
            package_name: &self.as_basic_data().package_name,
            version: &self.as_basic_data().version,
            slot: Slot {
                main: self.slot.main.as_str(),
                sub: self.slot.sub.as_str(),
            },
        }
    }

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
}

impl PackageDetails {
    pub fn as_basic_data(&self) -> &EBuildBasicData {
        &self.metadata.basic_data
    }

    pub fn as_metadata(&self) -> &EBuildMetadata {
        &self.metadata
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

        let package = ThinPackageRef {
            package_name: package_name.as_str(),
            version: &metadata.basic_data.version,
            slot: Slot {
                main: &slot.main,
                sub: &slot.sub,
            },
        };

        let raw_inherited = metadata.vars.get_scalar_or_default("INHERITED")?;
        let inherited: HashSet<String> = raw_inherited
            .split_ascii_whitespace()
            .map(|s| s.to_owned())
            .collect();

        let raw_inherit_paths = metadata.vars.get_indexed_array("INHERIT_PATHS")?;
        let inherit_paths: Vec<PathBuf> = raw_inherit_paths.iter().map(PathBuf::from).collect();

        let (accepted, stable) = match self.config.is_package_accepted(&metadata.vars, &package)? {
            IsPackageAcceptedResult::Unaccepted => {
                if self.force_accept_9999_ebuilds {
                    let accepted = inherited.contains("cros-workon")
                        && metadata.basic_data.version == self.version_9999
                        && match metadata.vars.get_scalar("CROS_WORKON_MANUAL_UPREV") {
                            Ok(value) => value != "1",
                            Err(_) => false,
                        };
                    (accepted, false)
                } else {
                    (false, false)
                }
            }
            IsPackageAcceptedResult::Accepted(stable) => (true, stable),
        };

        let iuse_map = parse_iuse_map(&metadata.vars)?;
        let use_map = self.config.compute_use_map(
            &package_name,
            &metadata.basic_data.version,
            stable,
            &slot,
            &iuse_map,
        );

        let required_use: RequiredUseDependency = metadata
            .vars
            .get_scalar_or_default("REQUIRED_USE")?
            .parse()?;

        let masked = !accepted
            || self.config.is_package_masked(&package)
            || required_use.matches(&use_map) == Some(false);

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

        Ok(PackageDetails {
            metadata,
            slot,
            use_map,
            accepted,
            stable,
            masked,
            inherited,
            inherit_paths,
            direct_build_target,
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

    use tempfile::TempDir;

    use crate::repository::{Repository, RepositorySet};

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

        let repo = Repository::new_for_testing("test", temp_dir);
        let repo_set = RepositorySet::new_for_testing(&[repo]);

        let evaluator = CachedEBuildEvaluator::new(
            repo_set.get_repos().into_iter().cloned().collect(),
            &temp_dir.join("tools"),
        );

        let config = ConfigBundle::new_empty_for_testing();
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
        assert_eq!(details.accepted, true);
        assert_eq!(details.stable, true);
        assert_eq!(details.masked, false);
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
        assert_eq!(details.accepted, false);
        assert_eq!(details.stable, false);
        assert_eq!(details.masked, true);
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
        assert_eq!(details.accepted, true);
        assert_eq!(details.stable, true);
        assert_eq!(details.masked, true);
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
}
