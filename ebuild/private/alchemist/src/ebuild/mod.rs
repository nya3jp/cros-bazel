// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod metadata;

use anyhow::Result;
use itertools::Itertools;
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
    dependency::package::{PackageRef, ThinPackageRef},
    repository::RepositorySet,
};

use self::metadata::EBuildEvaluator;

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

#[derive(Clone, Debug)]
pub struct PackageDetails {
    pub repo_name: String,
    pub package_name: String,
    pub version: Version,
    pub vars: BashVars,
    pub slot: Slot,
    pub use_map: UseMap,
    pub accepted: bool,
    pub stable: bool,
    pub masked: bool,
    pub ebuild_path: PathBuf,
    pub inherited: HashSet<String>,
}

impl PackageDetails {
    /// Converts this PackageDetails to a PackageRef that can be passed to
    /// dependency predicates.
    pub fn as_package_ref(&self) -> PackageRef {
        PackageRef {
            package_name: &self.package_name,
            version: &self.version,
            slot: Slot {
                main: self.slot.main.as_str(),
                sub: self.slot.sub.as_str(),
            },
            use_map: &self.use_map,
        }
    }

    pub fn as_thin_package_ref(&self) -> ThinPackageRef {
        ThinPackageRef {
            package_name: &self.package_name,
            version: &self.version,
            slot: Slot {
                main: self.slot.main.as_str(),
                sub: self.slot.sub.as_str(),
            },
        }
    }
}

#[derive(Debug)]
pub struct PackageLoader {
    repos: Arc<RepositorySet>,
    config: Arc<ConfigBundle>,
    evaluator: EBuildEvaluator,
    force_accept_9999_ebuilds: bool,
    version_9999: Version,
}

impl PackageLoader {
    pub fn new(
        repos: Arc<RepositorySet>,
        config: Arc<ConfigBundle>,
        tools_dir: &Path,
        force_accept_9999_ebuilds: bool,
    ) -> Self {
        let evaluator = EBuildEvaluator::new(tools_dir);
        Self {
            repos,
            config,
            evaluator,
            force_accept_9999_ebuilds,
            version_9999: Version::try_new("9999").unwrap(),
        }
    }

    pub fn load_package(&self, ebuild_path: &Path) -> Result<PackageDetails> {
        // Locate the repository this ebuild belongs to, which identifies
        // eclass directories to be available to the ebuild.
        let (repo, _) = self.repos.get_repo_by_path(ebuild_path)?;

        // Drive the ebuild to read its metadata.
        let metadata = self
            .evaluator
            .evaluate_metadata(ebuild_path, repo.eclass_dirs().collect_vec())?;

        // Compute additional information needed to fill in PackageDetails.
        let package_name = [
            metadata.path_info.category_name,
            metadata.path_info.package_short_name,
        ]
        .join("/");
        let slot = Slot::<String>::new(metadata.vars.get_scalar("SLOT")?);

        let package = ThinPackageRef {
            package_name: package_name.as_str(),
            version: &metadata.path_info.version,
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

        let (accepted, stable) = match self.config.is_package_accepted(&metadata.vars, &package)? {
            IsPackageAcceptedResult::Unaccepted => {
                if self.force_accept_9999_ebuilds {
                    let accepted = inherited.contains("cros-workon")
                        && metadata.path_info.version == self.version_9999
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
            &metadata.path_info.version,
            stable,
            &slot,
            &iuse_map,
        );

        let masked = !accepted || self.config.is_package_masked(&package);

        Ok(PackageDetails {
            repo_name: repo.name().to_string(),
            package_name,
            version: metadata.path_info.version,
            vars: metadata.vars,
            slot,
            use_map,
            accepted,
            stable,
            masked,
            inherited,
            ebuild_path: ebuild_path.to_owned(),
        })
    }
}

/// Wraps PackageLoader to cache results.
#[derive(Debug)]
pub struct CachedPackageLoader {
    loader: PackageLoader,
    cache: Mutex<HashMap<PathBuf, Arc<OnceCell<Arc<PackageDetails>>>>>,
}

impl CachedPackageLoader {
    pub fn new(loader: PackageLoader) -> Self {
        Self {
            loader,
            cache: Default::default(),
        }
    }

    pub fn load_package(&self, ebuild_path: &Path) -> Result<Arc<PackageDetails>> {
        let once_cell = {
            let mut cache_guard = self.cache.lock().unwrap();
            cache_guard
                .entry(ebuild_path.to_owned())
                .or_default()
                .clone()
        };
        let details =
            once_cell.get_or_try_init(|| self.loader.load_package(ebuild_path).map(Arc::new))?;
        Ok(details.clone())
    }
}
