// Copyright 2022 The ChromiumOS Authors.
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
    bash::BashVars,
    config::bundle::ConfigBundle,
    data::{IUseMap, PackageSlotKey, Slot, UseMap},
    dependency::package::PackageRef,
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

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Stability {
    Broken,
    Unknown,
    Unstable,
    Stable,
}

impl Stability {
    /// Computes the stability of a package according to variables defined by
    /// profiles and ebuild/eclasses.
    fn compute(vars: &BashVars, config: &ConfigBundle) -> Result<Self> {
        let arch = config.env().get("ARCH").map(|s| &**s).unwrap_or_default();

        let mut default_stability = Stability::Unknown;

        for keyword in vars
            .get_scalar_or_default("KEYWORDS")?
            .split_ascii_whitespace()
        {
            let (stability, trimed_keyword) = {
                if let Some(trimed_keyword) = keyword.strip_prefix('~') {
                    (Stability::Unstable, trimed_keyword)
                } else if let Some(trimed_keyword) = keyword.strip_prefix('-') {
                    (Stability::Broken, trimed_keyword)
                } else {
                    (Stability::Stable, keyword)
                }
            };
            if trimed_keyword == arch {
                return Ok(stability);
            }
            if trimed_keyword == "*" {
                default_stability = stability;
            }
        }
        Ok(default_stability)
    }
}

#[derive(Clone, Debug)]
pub struct PackageDetails {
    pub package_name: String,
    pub version: Version,
    pub vars: BashVars,
    pub slot: Slot,
    pub use_map: UseMap,
    pub stability: Stability,
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

    /// Returns a PackageSlotKey identifying a package name / main SLOT pair
    /// that this package occupies.
    pub fn slot_key(&self) -> PackageSlotKey {
        PackageSlotKey {
            package_name: self.package_name.clone(),
            main_slot: self.slot.main.clone(),
        }
    }
}

#[derive(Debug)]
pub struct PackageLoader {
    repos: RepositorySet,
    config: ConfigBundle,
    evaluator: EBuildEvaluator,
}

impl PackageLoader {
    pub fn new(repos: RepositorySet, config: ConfigBundle, tools_dir: &Path) -> Self {
        let evaluator = EBuildEvaluator::new(tools_dir);
        Self {
            repos,
            config,
            evaluator,
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
        let stability = Stability::compute(&metadata.vars, &self.config)?;
        let stable = stability == Stability::Stable;

        let slot = Slot::<String>::new(metadata.vars.get_scalar("SLOT")?);

        let iuse_map = parse_iuse_map(&metadata.vars)?;
        let use_map = self.config.compute_use_map(
            &package_name,
            &metadata.path_info.version,
            stable,
            &iuse_map,
        );

        let masked = self.config.is_package_masked(&PackageRef {
            package_name: package_name.as_str(),
            version: &metadata.path_info.version,
            slot: Slot {
                main: &slot.main,
                sub: &slot.sub,
            },
            use_map: &use_map,
        });

        let raw_inherited = metadata.vars.get_scalar_or_default("INHERITED")?;
        let inherited: HashSet<String> = raw_inherited
            .split_ascii_whitespace()
            .map(|s| s.to_owned())
            .collect();

        Ok(PackageDetails {
            package_name,
            version: metadata.path_info.version,
            vars: metadata.vars,
            slot,
            use_map,
            stability,
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
