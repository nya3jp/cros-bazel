// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Result};
use itertools::Itertools;
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
    dependency::{analyze_dependencies, PackageDependencies},
    source::{analyze_sources, PackageSources},
};

pub mod dependency;
pub mod restrict;
pub mod source;

/// Holds rich information about a package.
pub struct Package {
    /// Package information extracted by [`PackageResolver`].
    pub details: Arc<PackageDetails>,

    /// Dependency information computed from the package metadata.
    pub dependencies: PackageDependencies,

    /// Locates source code needed to build this package.
    pub sources: PackageSources,

    /// A list of packages needed to install together with this package.
    /// Specifically, it is a transitive closure of dependencies introduced by
    /// RDEPEND and PDEPEND. Alchemist needs to compute it, instead of letting
    /// Bazel compute it, because there can be circular dependencies.
    pub install_set: Vec<Arc<PackageDetails>>,

    /// The BDEPENDs declared by this package and all the IDEPENDs specified
    /// by the package's DEPENDs and their transitive RDEPENDs.
    ///
    /// When building the `build_deps` SDK layer, we need to ensure that all
    /// the IDEPENDs are installed into the `build_host_deps` SDK layer. We
    /// Could add the concept of an IDEPEND to bazel, but it would make the
    /// `sdk_install_deps` rule very complicated and harder to understand.
    pub build_host_deps: Vec<Arc<PackageDetails>>,

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
struct PackageLocalAnalysis {
    pub dependencies: PackageDependencies,
    pub sources: PackageSources,
    pub bashrcs: Vec<PathBuf>,
}

type MaybePackageLocalAnalysis = Result<Box<PackageLocalAnalysis>, Arc<PackageAnalysisError>>;

/// Results of package-global analysis, i.e. analysis that can be performed only after finishing
/// package-local analysis of all relevant packages.
struct PackageGlobalAnalysis {
    pub install_set: Vec<Arc<PackageDetails>>,
    pub build_host_deps: Vec<Arc<PackageDetails>>,
}

type MaybePackageGlobalAnalysis = Result<Box<PackageGlobalAnalysis>, Arc<PackageAnalysisError>>;

/// Performs DFS on the dependency graph presented by `local_map` and
/// records the install set of `current` to `install_map`. Note that
/// `install_map` is a [`HashMap`] because it is used for remembering visited
/// nodes.
fn find_install_map<'a>(
    local_map: &'a HashMap<PathBuf, MaybePackageLocalAnalysis>,
    current: &'a Arc<PackageDetails>,
    install_map: &mut HashMap<&'a Path, Arc<PackageDetails>>,
) {
    use std::collections::hash_map::Entry::*;
    match install_map.entry(current.as_basic_data().ebuild_path.as_path()) {
        Occupied(_) => {
            return;
        }
        Vacant(entry) => {
            entry.insert(current.clone());
        }
    }

    // PackageLocalAnalysis can be unavailable when analysis failed for the package
    // (e.g. failed to flatten RDEPEND). We can just skip traversing the graph in this case.
    // TODO: Handle analysis errors correctly.
    let local = match local_map
        .get(current.as_basic_data().ebuild_path.as_path())
        .expect("local_map is exhaustive")
    {
        Ok(local) => local,
        Err(_) => return,
    };

    let deps = &local.dependencies;
    let installs = deps.runtime_deps.iter().chain(deps.post_deps.iter());
    for install in installs {
        find_install_map(local_map, install, install_map);
    }
}

/// Returns the union of `current`'s `build_host_deps` and the
/// `install_host_deps` of all the `build_deps` and their transitive
/// `runtime_deps`.
fn compute_host_build_deps(
    local_map: &HashMap<PathBuf, MaybePackageLocalAnalysis>,
    current: &PackageDetails,
) -> Vec<Arc<PackageDetails>> {
    let mut build_dep_runtime_deps: HashMap<&Path, Arc<PackageDetails>> = HashMap::new();

    // PackageLocalAnalysis can be unavailable when analysis failed for the package
    // (e.g. failed to flatten RDEPEND). We can just return an empty dependencies in this case.
    // TODO: Handle analysis errors correctly.
    let local = match local_map
        .get(current.as_basic_data().ebuild_path.as_path())
        .expect("local_map is exhaustive")
    {
        Ok(local) => local,
        Err(_) => return Vec::new(),
    };

    for build_dep in &local.dependencies.build_deps {
        find_install_map(local_map, build_dep, &mut build_dep_runtime_deps);
    }

    build_dep_runtime_deps
        .into_values()
        .filter_map(|details| {
            local_map
                .get(details.as_basic_data().ebuild_path.as_path())
                .expect("local_map is exhaustive")
                .as_ref()
                .ok()
        })
        .flat_map(|local| &local.dependencies.install_host_deps)
        .chain(&local.dependencies.build_host_deps)
        .sorted_by_key(|details| &details.as_basic_data().ebuild_path)
        .unique_by(|details| &details.as_basic_data().ebuild_path)
        .cloned()
        .collect()
}

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
        let dependencies =
            analyze_dependencies(details, cross_compile, host_resolver, target_resolver)?;
        let sources = analyze_sources(config, details, src_dir)?;
        let bashrcs = config.package_bashrcs(&details.as_package_ref());
        Ok(PackageLocalAnalysis {
            dependencies,
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

    // Compute install sets.
    //
    // Portage provides two kinds of runtime dependencies: RDEPEND and PDEPEND.
    // They're very similar, but PDEPEND doesn't require dependencies to be
    // emerged in advance, and thus it's typically used to represent mutual
    // runtime dependencies without introducing circular dependencies.
    //
    // For example, sys-libs/pam and sys-auth/pambase depends on each other:
    // - sys-libs/pam:     PDEPEND="sys-auth/pambase"
    // - sys-auth/pambase: RDEPEND="sys-libs/pam"
    //
    // To build a ChromeOS base image, we need to build all packages depended
    // on for runtime by virtual/target-os, directly or indirectly. However,
    // we cannot simply represent PDEPEND as Bazel target dependencies since
    // they will introduce circular dependencies in Bazel dependency graph.
    // Therefore, alchemist needs to resolve PDEPEND and embed the computed
    // results in the generated BUILD.bazel files. Specifically, alchemist
    // needs to compute a transitive closure of a runtime dependency graph,
    // and to write the results as package_set Bazel targets.
    //
    // In the example above, sys-auth/pambase will appear in all package_set
    // targets that depend on it directly or indirectly, including sys-libs/pam
    // and virtual/target-os.
    //
    // There are some sophisticated algorithms to compute transitive closures,
    // but for our purpose it is sufficient to just traverse the dependency
    // graph starting from each node.
    let mut install_map: HashMap<&Path, Arc<PackageDetails>> = HashMap::new();
    find_install_map(local_map, details, &mut install_map);

    let install_set = install_map
        .into_values()
        .sorted_by(|a, b| {
            a.as_basic_data()
                .package_name
                .cmp(&b.as_basic_data().package_name)
                .then_with(|| a.as_basic_data().version.cmp(&b.as_basic_data().version))
        })
        .collect();

    let build_host_deps = compute_host_build_deps(local_map, details);

    Ok(Box::new(PackageGlobalAnalysis {
        install_set,
        build_host_deps,
    }))
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
                        dependencies: local.dependencies,
                        sources: local.sources,
                        bashrcs: local.bashrcs,
                        install_set: global.install_set,
                        build_host_deps: global.build_host_deps,
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
