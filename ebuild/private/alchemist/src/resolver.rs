// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use itertools::Itertools;
use rayon::prelude::*;
use std::sync::Arc;

use crate::{
    config::{bundle::ConfigBundle, ProvidedPackage},
    dependency::{package::PackageAtomDependency, Predicate},
    ebuild::{CachedEBuildEvaluator, PackageDetails, Stability},
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

/// Answers queries related to Portage packages.
#[derive(Debug)]
pub struct PackageResolver<'a> {
    repos: &'a RepositorySet,
    config: &'a ConfigBundle,
    evaluator: &'a CachedEBuildEvaluator,
    accept_stability: Stability,
}

impl<'a> PackageResolver<'a> {
    /// Constructs a new [`Resolver`].
    ///
    /// `accept_stability` specifies the minimum stability required for a
    /// package to be returned by `find_packages` and `find_best_package`.
    pub fn new(
        repos: &'a RepositorySet,
        config: &'a ConfigBundle,
        evaluator: &'a CachedEBuildEvaluator,
        accept_stability: Stability,
    ) -> Self {
        Self {
            repos,
            config,
            evaluator,
            accept_stability,
        }
    }

    /// Finds all packages matching the specified [`PackageAtomDependency`].
    pub fn find_packages(&self, atom: &PackageAtomDependency) -> Result<Vec<Arc<PackageDetails>>> {
        let ebuild_paths = self.repos.find_ebuilds(atom.package_name())?;

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
    pub fn find_provided_packages(
        &self,
        atom: &'a PackageAtomDependency,
    ) -> impl Iterator<Item = &'a ProvidedPackage> {
        self.config
            .provided_packages()
            .iter()
            .filter(|provided| atom.matches(&provided.as_thin_package_ref()))
    }
}
