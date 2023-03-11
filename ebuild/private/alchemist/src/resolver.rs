// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use itertools::Itertools;
use rayon::prelude::*;
use std::sync::Arc;
use version::Version;

use crate::{
    config::{bundle::ConfigBundle, ProvidedPackage},
    dependency::{
        package::{PackageAtom, PackageDependencyAtom},
        Predicate,
    },
    ebuild::{CachedPackageLoader, PackageDetails, Stability},
    repository::RepositorySet,
};

/// Answers queries related to Portage packages.
#[derive(Debug)]
pub struct PackageResolver<'a> {
    repos: &'a RepositorySet,
    config: &'a ConfigBundle,
    loader: &'a CachedPackageLoader,
    accept_stability: Stability,
    allow_9999_ebuilds: bool,
    version_9999: Version,
}

impl<'a> PackageResolver<'a> {
    /// Constructs a new [`Resolver`].
    ///
    /// `accept_stability` specifies the minimum stability required for a
    /// package to be returned by `find_packages` and `find_best_package`.
    ///
    /// `allow_9999_ebuilds` will consider 9999 cros-workon packages that don't
    /// specify CROS_WORKON_MANUAL_UPREV as stable.
    pub fn new(
        repos: &'a RepositorySet,
        config: &'a ConfigBundle,
        loader: &'a CachedPackageLoader,
        accept_stability: Stability,
        allow_9999_ebuilds: bool,
    ) -> Self {
        Self {
            repos,
            config,
            loader,
            accept_stability,
            allow_9999_ebuilds,
            version_9999: Version::try_new("9999").unwrap(),
        }
    }

    /// Finds all packages matching the specified [`PackageAtom`].
    ///
    /// Packages from a lower-priority repository come before packages from a
    /// higher-priority repository.
    pub fn find_packages(&self, atom: &PackageAtom) -> Result<Vec<Arc<PackageDetails>>> {
        let ebuild_paths = self.repos.find_ebuilds(atom.package_name())?;

        let packages = ebuild_paths
            .into_par_iter()
            .map(|ebuild_path| self.loader.load_package(&ebuild_path))
            .filter(|details| match details {
                Ok(details) => atom.matches(&details.as_thin_package_ref()),
                Err(_) => true,
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(packages)
    }

    /// Finds a package best matching the specified [`PackageAtomDependency`].
    ///
    /// If Ok(None) is returned that means that no suitable packages were found.
    /// If Err(_) is returned, that means there was an unexpected error looking
    /// for the package.
    pub fn find_best_package(
        &self,
        atom: &PackageDependencyAtom,
    ) -> Result<Option<Arc<PackageDetails>>> {
        let ebuild_paths = self.repos.find_ebuilds(atom.package_name())?;

        let packages = ebuild_paths
            .into_par_iter()
            .map(|ebuild_path| self.loader.load_package(&ebuild_path))
            .filter(|details| match details {
                Ok(details) => atom.matches(&details.as_package_ref()),
                Err(_) => true,
            })
            .collect::<Result<Vec<_>>>()?;

        self.find_best_package_in(&packages)
    }

    fn is_allowed_9999_ebuild(&self, package: &PackageDetails) -> bool {
        self.allow_9999_ebuilds
            && package.inherited.contains("cros-workon")
            && package.version == self.version_9999
            && match package.vars.get_scalar("CROS_WORKON_MANUAL_UPREV") {
                Ok(value) => value != "1",
                Err(_) => false,
            }
    }

    /// Finds the best package in the provided list.
    /// You must ensure all the packages have the same name.
    /// TODO(b/271000644): Define a PackageSelector.
    pub fn find_best_package_in(
        &self,
        packages: &[Arc<PackageDetails>],
    ) -> Result<Option<Arc<PackageDetails>>> {
        // Filter masked packages.
        let packages = packages
            .iter()
            .filter(|details| !details.masked)
            .collect_vec();

        // Select by stability.
        let packages = packages
            .into_iter()
            .filter(|details| {
                details.stability >= self.accept_stability || self.is_allowed_9999_ebuild(details)
            })
            .collect_vec();

        // Find the latest version.
        // max_by will return the last element if multiple elements are equal.
        // This translates to picking a package from an overlay with a higher
        // priority since the `packages` variable is sorted so that lower
        // priority packages come first and higher priority packages come last.
        Ok(packages
            .into_iter()
            .max_by(|a, b| a.version.cmp(&b.version))
            .cloned())
    }

    /// Finds *provided packages* matching the specified [`PackageAtomDependency`].
    ///
    /// Portage allows pretending a missing package as "provided" by configuring
    /// `package.provided`. This method allows accessing the list.
    pub fn find_provided_packages(
        &self,
        atom: &'a PackageDependencyAtom,
    ) -> impl Iterator<Item = &'a ProvidedPackage> {
        self.config
            .provided_packages()
            .iter()
            .filter(|provided| atom.matches(provided))
    }
}
