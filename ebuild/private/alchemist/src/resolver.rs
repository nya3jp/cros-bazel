// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use rayon::prelude::*;
use std::sync::Arc;
use version::Version;

use crate::{
    config::{bundle::ConfigBundle, ProvidedPackage},
    data::UseMap,
    dependency::{
        package::{PackageAtom, PackageDependencyAtom},
        Predicate,
    },
    ebuild::{CachedPackageLoader, PackageDetails},
    repository::RepositorySet,
};

/// Answers queries related to Portage packages.
#[derive(Debug)]
pub struct PackageResolver {
    repos: Arc<RepositorySet>,
    config: Arc<ConfigBundle>,
    loader: Arc<CachedPackageLoader>,
}

impl PackageResolver {
    /// Constructs a new [`Resolver`].
    pub fn new(
        repos: Arc<RepositorySet>,
        config: Arc<ConfigBundle>,
        loader: Arc<CachedPackageLoader>,
    ) -> Self {
        Self {
            repos,
            config,
            loader,
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

    /// Finds the best package matching the specified [`PackageAtom`].
    pub fn find_best_package(&self, atom: &PackageAtom) -> Result<Option<Arc<PackageDetails>>> {
        let matches = self
            .find_packages(atom)
            .with_context(|| format!("Error looking up {atom}"))?;
        self.find_best_package_in(&matches)
    }

    /// Finds a package best matching the specified [`PackageAtomDependency`].
    ///
    /// # Arguments
    ///
    /// * `use_map` - The [`UseMap`] for the package that specified the `atom`.
    /// * `atom` - The `atom` used to filter the packages.
    ///
    /// If Ok(None) is returned that means that no suitable packages were found.
    /// If Err(_) is returned, that means there was an unexpected error looking
    /// for the package.
    pub fn find_best_package_dependency(
        &self,
        use_map: &UseMap,
        atom: &PackageDependencyAtom,
    ) -> Result<Option<Arc<PackageDetails>>> {
        let ebuild_paths = self.repos.find_ebuilds(atom.package_name())?;

        let packages = ebuild_paths
            .into_par_iter()
            .map(|ebuild_path| self.loader.load_package(&ebuild_path))
            .collect::<Result<Vec<_>>>()?;

        let mut matches = Vec::with_capacity(packages.len());
        for details in packages {
            match atom.package_matches(use_map, &details.as_package_ref()) {
                Ok(result) => {
                    if result {
                        matches.push(details);
                    }
                }
                // We don't use with_context because we want to manually format
                // the error.
                Err(err) => bail!(
                    "target: {}-{}: {}",
                    details.package_name,
                    details.version,
                    err
                ),
            }
        }

        self.find_best_package_in(&matches)
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
    pub fn find_provided_packages<'a>(
        &'a self,
        atom: &'a PackageDependencyAtom,
    ) -> impl Iterator<Item = &'a ProvidedPackage> {
        self.config
            .provided_packages()
            .iter()
            .filter(|provided| atom.matches(provided))
    }

    /// Checks if the package is provided.
    pub fn is_provided(&self, package_name: &str, version: &Version) -> bool {
        self.config
            .provided_packages()
            .iter()
            .any(|provided| provided.package_name == package_name && &provided.version == version)
    }
}
