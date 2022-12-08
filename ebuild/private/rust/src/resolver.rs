// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use itertools::Itertools;
use rayon::prelude::*;
use std::{path::Path, sync::Arc};

use crate::{
    config::{site::SiteSettings, ConfigBundle, ConfigSource, ProvidedPackage},
    data::Vars,
    dependency::{package::PackageAtomDependency, Predicate},
    ebuild::{CachedEBuildEvaluator, EBuildEvaluator, PackageDetails, Stability},
    profile::Profile,
    repository::RepositorySet,
};

/// An error returned by `Resolver::find_best_package`.
///
/// Particuarly it allows checking if the method failed because no package
/// satisfies the given package dependency atom.
#[derive(Debug, thiserror::Error)]
pub enum FindBestPackageError {
    #[error("package not found")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// A convenient wrapper of various types, such as [`RepositorySet`] and
/// [`EBuildEvaluator`], to provide handy access to Portage tree information.
#[derive(Debug)]
pub struct Resolver {
    config: ConfigBundle,
    repos: RepositorySet,
    evaluator: CachedEBuildEvaluator,
    accept_stability: Stability,
}

impl Resolver {
    /// Constructs a new [`Resolver`] by loading configurations from the
    /// specified configuration root directory.
    ///
    /// `tools_dir` is a path to the directory containing tool binaries needed
    /// to evaluate ebuilds, such as `ver_test`.
    ///
    /// `accept_stability` specifies the minimum stability required for a
    /// package to be returned by `find_packages` and `find_best_package`.
    pub fn load(root_dir: &Path, tools_dir: &Path, accept_stability: Stability) -> Result<Self> {
        let repos = RepositorySet::load(root_dir)?;
        let profile = Profile::load_default(root_dir, &repos)?;

        let site_settings = SiteSettings::load(root_dir)?;

        let mut env = Vars::new();
        let profile_nodes = profile.evaluate_configs(&mut env);
        let site_settings_nodes = site_settings.evaluate_configs(&mut env);
        let all_nodes = profile_nodes
            .into_iter()
            .chain(site_settings_nodes.into_iter())
            .collect();
        let config = ConfigBundle::new(env, all_nodes);

        // TODO: Avoid cloning ConfigBundle.
        let evaluator = CachedEBuildEvaluator::new(EBuildEvaluator::new(
            repos.clone(),
            config.clone(),
            tools_dir,
        ));
        Ok(Self {
            config,
            repos,
            evaluator,
            accept_stability,
        })
    }

    /// Finds all packages matching the specified [`PackageAtomDependency`].
    pub fn find_packages(&self, atom: &PackageAtomDependency) -> Result<Vec<Arc<PackageDetails>>> {
        let ebuild_paths = self.repos.scan_ebuilds(atom.package_name())?;

        let mut packages = ebuild_paths
            .into_par_iter()
            .map(|ebuild_path| self.evaluator.evaluate(&ebuild_path))
            .filter(|details| match details {
                Ok(details) => atom.matches(&details.as_package_ref()),
                Err(_) => true,
            })
            .collect::<Result<Vec<_>>>()?;
        packages.sort_unstable_by_key(|package| package.version.clone());
        packages.reverse();
        Ok(packages)
    }

    /// Finds a package best matching the specified [`PackageAtomDependency`].
    pub fn find_best_package(
        &self,
        atom: &PackageAtomDependency,
    ) -> std::result::Result<Arc<PackageDetails>, FindBestPackageError> {
        let packages = self.find_packages(atom)?;

        // Filter masked packages.
        let packages = packages
            .into_iter()
            .filter(|details| !details.masked)
            .collect_vec();

        // Select by stability.
        let packages = packages
            .into_iter()
            .filter(|details| details.stability >= self.accept_stability)
            .collect_vec();

        // Find the latest version.
        packages
            .into_iter()
            .max_by(|a, b| a.version.cmp(&b.version))
            .ok_or(FindBestPackageError::NotFound)
    }

    /// Finds *provided packages* matching the specified [`PackageAtomDependency`].
    ///
    /// Portage allows pretending a missing package as "provided" by configuring
    /// `package.provided`. This method allows accessing the list.
    pub fn find_provided_packages<'a>(
        &'a self,
        atom: &'a PackageAtomDependency,
    ) -> impl Iterator<Item = &'a ProvidedPackage> {
        self.config
            .provided_packages()
            .iter()
            .filter(|provided| atom.matches(&provided.as_thin_package_ref()))
    }
}
