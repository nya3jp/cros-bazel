// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use std::sync::Arc;
use tracing::instrument;
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

    /// Loads all packages covered by this resolver.
    #[instrument(skip_all)]
    pub fn find_all_packages(&self) -> Result<Vec<MaybePackageDetails>> {
        // Load packages in parallel.
        self.repos
            .find_all_ebuilds()?
            .into_par_iter()
            .map(|ebuild_path| self.loader.load_package(&ebuild_path))
            .collect()
    }

    /// Finds all packages matching the specified [`PackageAtom`].
    ///
    /// It returns both unmasked and masked packages as long as they match the given atom.
    ///
    /// Packages from a lower-priority repository come before packages from a higher-priority
    /// repository, which is the suitable order for [`select_best_version`].
    pub fn find_packages(&self, atom: &PackageAtom) -> Result<Vec<MaybePackageDetails>> {
        let ebuild_paths = self.repos.find_ebuilds(atom.package_name())?;

        let packages = ebuild_paths
            .into_par_iter()
            .filter_map(|ebuild_path| match self.loader.load_package(&ebuild_path) {
                Ok(maybe_details) if !atom.matches(&maybe_details.as_package_ref()) => None,
                other => Some(other),
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(packages)
    }

    /// Finds the valid package best matching the specified [`PackageAtom`].
    ///
    /// It returns `Ok(Some(_))` if it can reliably determine the best package. Note that it might
    /// be possible to determine the best package even if some candidate packages have load errors.
    /// It returns `Ok(None)` if no package matches the given atom. It returns `Err` otherwise.
    pub fn find_best_package(&self, atom: &PackageAtom) -> Result<Option<Arc<PackageDetails>>> {
        let matches = self
            .find_packages(atom)
            .with_context(|| format!("Error looking up {atom}"))?;
        match select_best_version(&matches) {
            Some(MaybePackageDetails::Ok(details)) => Ok(Some(details.clone())),
            None => Ok(None),
            Some(MaybePackageDetails::Err(err)) => bail!(
                "Cannot determine the best version for {}: {}-{}: {}",
                atom,
                err.as_basic_data().package_name,
                err.as_basic_data().version,
                err.error
            ),
        }
    }

    /// Finds the valid package best matching the specified [`PackageDependencyAtom`].
    ///
    /// # Arguments
    ///
    /// * `source_use_map` - The [`UseMap`] for the package that specified the `atom`.
    /// * `atom` - The atom used to filter the packages.
    ///
    /// # Returns
    ///
    /// It returns `Ok(Some(_))` if it can reliably determine the best package. Note that it might
    /// be possible to determine the best package even if some candidate packages have load errors.
    /// It returns `Ok(None)` if no package matches the given dependency atom. It returns `Err`
    /// otherwise.
    pub fn find_best_package_dependency(
        &self,
        source_use_map: &UseMap,
        atom: &PackageDependencyAtom,
    ) -> Result<Option<Arc<PackageDetails>>> {
        let ebuild_paths = self.repos.find_ebuilds(atom.package_name())?;

        let packages = ebuild_paths
            .into_par_iter()
            .map(|ebuild_path| -> Result<Option<MaybePackageDetails>> {
                let maybe_details = self.loader.load_package(&ebuild_path)?;
                // TODO: Make match errors non-fatal.
                if atom.matches(source_use_map, &maybe_details.as_package_ref())? {
                    Ok(Some(maybe_details))
                } else {
                    Ok(None)
                }
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        match select_best_version(&packages) {
            Some(MaybePackageDetails::Ok(details)) => Ok(Some(details.clone())),
            None => Ok(None),
            Some(MaybePackageDetails::Err(err)) => bail!(
                "Cannot determine the best version for {}: {}-{}: {}",
                atom,
                err.as_basic_data().package_name,
                err.as_basic_data().version,
                err.error
            ),
        }
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
