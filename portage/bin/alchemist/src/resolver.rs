// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use std::sync::Arc;
use version::Version;

use crate::{
    config::{bundle::ConfigBundle, ProvidedPackage},
    data::UseMap,
    dependency::{
        package::{AsPackageRef, PackageAtom, PackageDependencyAtom},
        Predicate,
    },
    ebuild::{CachedPackageLoader, MaybePackageDetails, PackageDetails},
    repository::RepositorySet,
};

/// Selects the best version from the provided list of packages.
///
/// ## Notes
///
/// - The provided list must contain the same package only.
/// - Packages known to be masked are excluded from consideration.
/// - In the case there are two or more tied candidate packages, the one appears the last in the
///   list is selected.
/// - It may return a package that failed to load/analyze.
pub fn select_best_version<T: AsPackageRef, I: IntoIterator<Item = T>>(packages: I) -> Option<T> {
    // max_by will return the last element if multiple elements are equal.
    packages
        .into_iter()
        .filter(|p| p.as_package_ref().readiness != Some(false))
        .max_by(|a, b| a.as_package_ref().version.cmp(b.as_package_ref().version))
}

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
    /// Packages from a lower-priority repository come before packages from a higher-priority
    /// repository, which is the suitable order for [`select_best_version`].
    pub fn find_packages(&self, atom: &PackageAtom) -> Result<Vec<Arc<PackageDetails>>> {
        let ebuild_paths = self.repos.find_ebuilds(atom.package_name())?;

        let packages = ebuild_paths
            .into_par_iter()
            .map(|ebuild_path| self.loader.load_package(&ebuild_path))
            .filter_map(|result| match result {
                Ok(eval) => match eval {
                    MaybePackageDetails::Ok(details) => Some(Ok(details)),
                    // We ignore packages that had metadata evaluation errors.
                    MaybePackageDetails::Err(_) => None,
                },
                Err(e) => Some(Err(e)),
            })
            .filter(|details| match details {
                Ok(details) => atom.matches(&details.as_package_ref()),
                Err(_) => true,
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(packages)
    }

    /// Finds the best package matching the specified [`PackageAtom`].
    ///
    /// It returns `Ok(None)` if there is no matching packages, `Ok(Some(_))` if it can determine
    /// the best package, `Err` otherwise.
    pub fn find_best_package(&self, atom: &PackageAtom) -> Result<Option<Arc<PackageDetails>>> {
        let matches = self
            .find_packages(atom)
            .with_context(|| format!("Error looking up {atom}"))?;
        Ok(select_best_version(matches))
    }

    /// Finds a package best matching the specified [`PackageAtomDependency`].
    ///
    /// # Arguments
    ///
    /// * `source_use_map` - The [`UseMap`] for the package that specified the `atom`.
    /// * `atom` - The atom used to filter the packages.
    ///
    /// If Ok(None) is returned that means that no suitable packages were found.
    /// If Err(_) is returned, that means there was an unexpected error looking
    /// for the package.
    pub fn find_best_package_dependency(
        &self,
        source_use_map: &UseMap,
        atom: &PackageDependencyAtom,
    ) -> Result<Option<Arc<PackageDetails>>> {
        let ebuild_paths = self.repos.find_ebuilds(atom.package_name())?;

        let packages = ebuild_paths
            .into_par_iter()
            .map(|ebuild_path| self.loader.load_package(&ebuild_path))
            .collect::<Result<Vec<_>>>()?;
        let mut matches = Vec::with_capacity(packages.len());
        for eval in packages {
            let details = match eval {
                MaybePackageDetails::Ok(details) => details,
                // We ignore packages that had metadata evaluation errors.
                MaybePackageDetails::Err(_) => continue,
            };
            match atom.matches(source_use_map, &details.as_package_ref()) {
                Ok(result) => {
                    if result {
                        matches.push(details);
                    }
                }
                // We don't use with_context because we want to manually format
                // the error.
                Err(err) => bail!(
                    "target: {}-{}: {}",
                    details.as_basic_data().package_name,
                    details.as_basic_data().version,
                    err
                ),
            }
        }

        Ok(select_best_version(matches))
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
            .filter(|provided| atom.matches_provided(provided))
    }

    /// Checks if the package is provided.
    pub fn is_provided(&self, package_name: &str, version: &Version) -> bool {
        self.config
            .provided_packages()
            .iter()
            .any(|provided| provided.package_name == package_name && &provided.version == version)
    }
}
