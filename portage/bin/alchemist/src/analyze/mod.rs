// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Result};
use rayon::prelude::*;
use tracing::instrument;

use crate::{
    config::bundle::ConfigBundle,
    dependency::package::{AsPackageRef, PackageRef},
    ebuild::{
        metadata::{EBuildBasicData, EBuildMetadata},
        MaybePackageDetails, PackageDetails, PackageReadiness,
    },
    resolver::PackageResolver,
};

use self::{
    dependency::{
        direct::{analyze_direct_dependencies, DependencyExpressions, DirectDependencies},
        indirect::{analyze_indirect_dependencies, IndirectDependencies},
    },
    source::{analyze_sources, PackageSources},
};

pub mod dependency;
pub mod restrict;
pub mod source;
#[cfg(test)]
mod tests;

pub struct PackageDependencies {
    pub direct: DirectDependencies,
    pub expressions: DependencyExpressions,
    pub indirect: IndirectDependencies,
}

/// Holds rich information about a package.
pub struct Package {
    /// Package information extracted by [`PackageResolver`].
    pub details: Arc<PackageDetails>,

    /// Dependency information computed from the package metadata.
    pub dependencies: PackageDependencies,

    /// Locates source code needed to build this package.
    pub sources: PackageSources,

    /// The bashrc files that need to be executed for the package.
    ///
    /// This list contains relevant profile.bashrc and the package specific
    /// bashrc files defined by package.bashrc. They are ordered in the sequence
    /// that they should be executed.
    pub bashrcs: Vec<PathBuf>,
}

#[allow(dead_code)]
impl Package {
    pub fn as_basic_data(&self) -> &EBuildBasicData {
        &self.details.metadata.basic_data
    }

    pub fn as_metadata(&self) -> &EBuildMetadata {
        &self.details.metadata
    }

    pub fn as_details(&self) -> &PackageDetails {
        &self.details
    }
}

impl AsPackageRef for Package {
    fn as_package_ref(&self) -> PackageRef {
        self.details.as_package_ref()
    }
}

impl AsRef<DirectDependencies> for Package {
    fn as_ref(&self) -> &DirectDependencies {
        &self.dependencies.direct
    }
}

/// Holds information for packages that we failed to analyze.
#[derive(Debug)]
pub struct PackageAnalysisError {
    pub details: MaybePackageDetails,
    pub error: String,
}

impl PackageAnalysisError {
    pub fn as_basic_data(&self) -> &EBuildBasicData {
        self.details.as_basic_data()
    }
}

impl AsPackageRef for PackageAnalysisError {
    fn as_package_ref(&self) -> PackageRef {
        self.details.as_package_ref()
    }
}

/// Represents a package, covering both successfully analyzed ones and failed ones.
///
/// Since this enum is very lightweight (contains [`Arc`] only), you should not wrap it within
/// reference-counting smart pointers like [`Arc`], but you can just clone it.
///
/// While this enum looks very similar to [`Result`], we don't make it a type alias of [`Result`]
/// to implement a few convenient methods.
#[derive(Clone)]
pub enum MaybePackage {
    Ok(Arc<Package>),
    Err(Arc<PackageAnalysisError>),
}

impl<'a> From<&'a MaybePackage> for Result<&'a Package, &'a PackageAnalysisError> {
    fn from(value: &'a MaybePackage) -> Self {
        match value {
            MaybePackage::Ok(package) => Ok(package.as_ref()),
            MaybePackage::Err(err) => Err(err.as_ref()),
        }
    }
}

impl MaybePackage {
    pub fn as_basic_data(&self) -> &EBuildBasicData {
        match self {
            MaybePackage::Ok(package) => package.as_basic_data(),
            MaybePackage::Err(error) => error.as_basic_data(),
        }
    }
}

impl AsPackageRef for MaybePackage {
    fn as_package_ref(&self) -> PackageRef {
        match self {
            MaybePackage::Ok(package) => package.as_package_ref(),
            MaybePackage::Err(error) => error.as_package_ref(),
        }
    }
}

/// Results of package-local analysis, i.e. analysis that can be performed independently of other
/// packages.
pub struct PackageLocalAnalysis {
    pub direct_dependencies: DirectDependencies,
    pub expressions: DependencyExpressions,
    pub sources: PackageSources,
    pub bashrcs: Vec<PathBuf>,
}

impl AsRef<DirectDependencies> for PackageLocalAnalysis {
    fn as_ref(&self) -> &DirectDependencies {
        &self.direct_dependencies
    }
}

pub type MaybePackageLocalAnalysis = Result<Box<PackageLocalAnalysis>, Arc<PackageAnalysisError>>;

/// Results of package-global analysis, i.e. analysis that can be performed only after finishing
/// package-local analysis of all relevant packages.
pub struct PackageGlobalAnalysis {
    pub indirect_dependencies: IndirectDependencies,
}

pub type MaybePackageGlobalAnalysis = Result<Box<PackageGlobalAnalysis>, Arc<PackageAnalysisError>>;

fn analyze_local(
    details: &MaybePackageDetails,
    config: &ConfigBundle,
    cross_compile: bool,
    src_dir: &Path,
    host_resolver: &PackageResolver,
    target_resolver: &PackageResolver,
) -> MaybePackageLocalAnalysis {
    let details = match details {
        MaybePackageDetails::Ok(details) => details,
        MaybePackageDetails::Err(error) => {
            return Err(Arc::new(PackageAnalysisError {
                error: error.error.clone(),
                details: MaybePackageDetails::Err(error.clone()),
            }));
        }
    };
    let result = (|| -> Result<PackageLocalAnalysis> {
        if let PackageReadiness::Masked { reason } = &details.readiness {
            // We do not support building masked packages because of
            // edge cases: e.g., if one masked package depends on
            // another masked one, this'd be treated as an unsatisfied
            // dependency error.
            bail!("The package is masked: {}", reason);
        }
        let (direct_dependencies, expressions) =
            analyze_direct_dependencies(details, cross_compile, host_resolver, target_resolver)?;
        let sources = analyze_sources(config, details, src_dir)?;
        let bashrcs = config.package_bashrcs(&details.as_package_ref());
        Ok(PackageLocalAnalysis {
            direct_dependencies,
            expressions,
            sources,
            bashrcs,
        })
    })();
    match result {
        Ok(local) => Ok(Box::new(local)),
        Err(err) => Err(Arc::new(PackageAnalysisError {
            details: MaybePackageDetails::Ok(details.clone()),
            error: format!("{err:#}"),
        })),
    }
}

/// Runs package-local analysis, i.e. analysis that can be done independently of other packages.
fn analyze_locals(
    all_details: &[MaybePackageDetails],
    config: &ConfigBundle,
    cross_compile: bool,
    src_dir: &Path,
    host_resolver: &PackageResolver,
    target_resolver: &PackageResolver,
) -> HashMap<PathBuf, MaybePackageLocalAnalysis> {
    // Analyze packages in parallel.
    all_details
        .into_par_iter()
        .map(|details| {
            let local = analyze_local(
                details,
                config,
                cross_compile,
                src_dir,
                host_resolver,
                target_resolver,
            );
            (details.as_basic_data().ebuild_path.clone(), local)
        })
        .collect()
}

fn analyze_global(
    details: &MaybePackageDetails,
    local_map: &HashMap<PathBuf, MaybePackageLocalAnalysis>,
) -> MaybePackageGlobalAnalysis {
    let details = match details {
        MaybePackageDetails::Ok(details) => details,
        MaybePackageDetails::Err(error) => {
            return Err(Arc::new(PackageAnalysisError {
                details: details.clone(),
                error: error.error.clone(),
            }))
        }
    };

    let result = (|| -> Result<PackageGlobalAnalysis> {
        let indirect_dependencies = analyze_indirect_dependencies(details, local_map)?;
        Ok(PackageGlobalAnalysis {
            indirect_dependencies,
        })
    })();

    match result {
        Ok(global) => Ok(Box::new(global)),
        Err(error) => Err(Arc::new(PackageAnalysisError {
            details: MaybePackageDetails::Ok(details.clone()),
            error: format!("{error:#}"),
        })),
    }
}

/// Runs package-global analysis, i.e. analysis taking other packages into account.
fn analyze_globals(
    all_details: &[MaybePackageDetails],
    local_map: &HashMap<PathBuf, MaybePackageLocalAnalysis>,
) -> HashMap<PathBuf, MaybePackageGlobalAnalysis> {
    all_details
        .into_par_iter()
        .map(|details| {
            let global = analyze_global(details, local_map);
            (details.as_basic_data().ebuild_path.clone(), global)
        })
        .collect()
}

#[instrument(skip_all)]
pub fn analyze_packages(
    config: &ConfigBundle,
    cross_compile: bool,
    src_dir: &Path,
    host_resolver: &PackageResolver,
    target_resolver: &PackageResolver,
) -> Result<Vec<MaybePackage>> {
    // Load all packages.
    let all_details = target_resolver.find_all_packages()?;

    // Run package-local analysis.
    let mut local_map = analyze_locals(
        &all_details,
        config,
        cross_compile,
        src_dir,
        host_resolver,
        target_resolver,
    );

    // Run package-global analysis.
    let mut global_map = analyze_globals(&all_details, &local_map);

    // Join analysis results.
    let packages: Vec<MaybePackage> = all_details
        .into_iter()
        .map(|details| {
            let ebuild_path = details.as_basic_data().ebuild_path.as_path();
            let local = local_map
                .remove(ebuild_path)
                .expect("local_map is exhaustive");
            let global = global_map
                .remove(ebuild_path)
                .expect("global_map is exhaustive");
            match (details, local, global) {
                (MaybePackageDetails::Ok(details), Ok(local), Ok(global)) => {
                    MaybePackage::Ok(Arc::new(Package {
                        details,
                        dependencies: PackageDependencies {
                            direct: local.direct_dependencies,
                            expressions: local.expressions,
                            indirect: global.indirect_dependencies,
                        },
                        sources: local.sources,
                        bashrcs: local.bashrcs,
                    }))
                }
                (MaybePackageDetails::Err(error), _, _) => {
                    MaybePackage::Err(Arc::new(PackageAnalysisError {
                        details: MaybePackageDetails::Err(error.clone()),
                        error: error.error.clone(),
                    }))
                }
                (_, Err(error), _) => MaybePackage::Err(error),
                (_, _, Err(error)) => MaybePackage::Err(error),
            }
        })
        .collect();

    let errors = packages
        .iter()
        .filter(|p| matches!(p, MaybePackage::Err(_)))
        .count();
    if errors > 0 {
        eprintln!("WARNING: Analysis failed for {} packages", errors);
    }

    Ok(packages)
}
