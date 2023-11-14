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

/// Similar to [`Package`], but an install set is not resolved yet.
struct PackagePartial {
    pub details: Arc<PackageDetails>,
    pub dependencies: PackageDependencies,
    pub sources: PackageSources,
}

#[allow(dead_code)]
impl PackagePartial {
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

/// Represents a partially analyzed package, covering both successful ones and failed ones.
///
/// While this enum looks very similar to [`Result`], we don't make it a type alias of [`Result`]
/// to implement a few convenient methods.
enum MaybePackagePartial {
    Ok(Box<PackagePartial>),
    Err(Arc<PackageAnalysisError>),
}

#[allow(dead_code)]
impl MaybePackagePartial {
    pub fn as_basic_data(&self) -> &EBuildBasicData {
        match self {
            MaybePackagePartial::Ok(package) => package.as_basic_data(),
            MaybePackagePartial::Err(error) => error.as_basic_data(),
        }
    }
}

/// Performs DFS on the dependency graph presented by `partial_by_path` and
/// records the install set of `current` to `install_map`. Note that
/// `install_map` is a [`HashMap`] because it is used for remembering visited
/// nodes.
fn find_install_map<'a>(
    partial_by_path: &'a HashMap<&Path, &PackagePartial>,
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

    // PackagePartial can be unavailable when analysis failed for the package
    // (e.g. failed to flatten RDEPEND). We can just skip traversing the graph
    // in this case.
    let current_partial = match partial_by_path.get(current.as_basic_data().ebuild_path.as_path()) {
        Some(partial) => partial,
        None => {
            return;
        }
    };

    let deps = &current_partial.dependencies;
    let installs = deps.runtime_deps.iter().chain(deps.post_deps.iter());
    for install in installs {
        find_install_map(partial_by_path, install, install_map);
    }
}

/// Adds `current` and all of `current`'s runtime deps into to `runtime_deps`.
fn collect_runtime_deps<'a>(
    partial_by_path: &'a HashMap<&Path, &PackagePartial>,
    current: &'a Arc<PackageDetails>,
    runtime_deps: &mut HashMap<&'a Path, Arc<PackageDetails>>,
) {
    use std::collections::hash_map::Entry::*;
    match runtime_deps.entry(current.as_basic_data().ebuild_path.as_path()) {
        Occupied(_) => {
            return;
        }
        Vacant(entry) => {
            entry.insert(current.clone());
        }
    }

    // PackagePartial can be unavailable when analysis failed for the package
    // (e.g. failed to flatten RDEPEND). We can just skip traversing the graph
    // in this case.
    let current_partial = match partial_by_path.get(current.as_basic_data().ebuild_path.as_path()) {
        Some(partial) => partial,
        None => {
            return;
        }
    };

    let deps = &current_partial.dependencies;
    // TODO(rrangel): Profile this and see if we should instead cache the
    // computed RDEPENDs instead of traversing the graph every call.
    for runtime_dep in &deps.runtime_deps {
        collect_runtime_deps(partial_by_path, runtime_dep, runtime_deps);
    }
}

/// Returns the union of `current`'s `build_host_deps` and the
/// `install_host_deps` of all the `build_deps` and their transitive
/// `runtime_deps`.
fn compute_host_build_deps<'a>(
    partial_by_path: &'a HashMap<&Path, &PackagePartial>,
    current: &'a PackagePartial,
) -> Vec<Arc<PackageDetails>> {
    let mut build_dep_runtime_deps: HashMap<&'a Path, Arc<PackageDetails>> = HashMap::new();

    for build_dep in &current.dependencies.build_deps {
        collect_runtime_deps(partial_by_path, build_dep, &mut build_dep_runtime_deps);
    }

    build_dep_runtime_deps
        .into_values()
        .filter_map(|details| partial_by_path.get(details.as_basic_data().ebuild_path.as_path()))
        .flat_map(|partial| &partial.dependencies.install_host_deps)
        .chain(&current.dependencies.build_host_deps)
        .sorted_by_key(|details| &details.as_basic_data().ebuild_path)
        .unique_by(|details| &details.as_basic_data().ebuild_path)
        .cloned()
        .collect()
}

#[instrument(skip_all)]
pub fn analyze_packages(
    config: &ConfigBundle,
    cross_compile: bool,
    all_details: Vec<MaybePackageDetails>,
    src_dir: &Path,
    host_resolver: Option<&PackageResolver>,
    target_resolver: &PackageResolver,
) -> Vec<MaybePackage> {
    // Analyze packages in parallel.
    let all_partials: Vec<MaybePackagePartial> = all_details
        .into_par_iter()
        .map(|details| {
            let details = match details {
                MaybePackageDetails::Ok(details) => details,
                MaybePackageDetails::Err(error) => {
                    return MaybePackagePartial::Err(Arc::new(PackageAnalysisError {
                        error: error.error.clone(),
                        details: MaybePackageDetails::Err(error),
                    }));
                }
            };
            let result = (|| -> Result<PackagePartial> {
                if let PackageReadiness::Masked { reason } = &details.readiness {
                    // We do not support building masked packages because of
                    // edge cases: e.g., if one masked package depends on
                    // another masked one, this'd be treated as an unsatisfied
                    // dependency error.
                    bail!("The package is masked: {}", reason);
                }
                let dependencies =
                    analyze_dependencies(&details, cross_compile, host_resolver, target_resolver)?;
                let sources = analyze_sources(config, &details, src_dir)?;
                Ok(PackagePartial {
                    details: details.clone(),
                    dependencies,
                    sources,
                })
            })();
            match result {
                Ok(package) => MaybePackagePartial::Ok(Box::new(package)),
                Err(err) => MaybePackagePartial::Err(Arc::new(PackageAnalysisError {
                    details: MaybePackageDetails::Ok(details),
                    error: format!("{err:#}"),
                })),
            }
        })
        .collect();

    let errors = all_partials
        .iter()
        .filter(|p| matches!(p, MaybePackagePartial::Err(_)))
        .count();
    if errors > 0 {
        eprintln!("WARNING: Analysis failed for {} packages", errors);
    }

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

    let partial_by_path: HashMap<&Path, &PackagePartial> = all_partials
        .iter()
        .flat_map(|partial| match partial {
            MaybePackagePartial::Ok(partial) => Some(partial.as_ref()),
            _ => None,
        })
        .map(|partial| (partial.as_basic_data().ebuild_path.as_path(), partial))
        .collect();

    let mut install_set_by_path: HashMap<PathBuf, Vec<Arc<PackageDetails>>> = partial_by_path
        .iter()
        .map(|(path, partial)| {
            let mut install_map: HashMap<&Path, Arc<PackageDetails>> = HashMap::new();
            find_install_map(&partial_by_path, &partial.details, &mut install_map);

            let install_set = install_map
                .into_values()
                .sorted_by(|a, b| {
                    a.as_basic_data()
                        .package_name
                        .cmp(&b.as_basic_data().package_name)
                        .then_with(|| a.as_basic_data().version.cmp(&b.as_basic_data().version))
                })
                .collect();

            ((*path).to_owned(), install_set)
        })
        .collect();

    let mut build_host_deps_by_path: HashMap<PathBuf, Vec<Arc<PackageDetails>>> = partial_by_path
        .iter()
        .map(|(path, partial)| {
            (
                path.to_path_buf(),
                compute_host_build_deps(&partial_by_path, partial),
            )
        })
        .collect();

    let packages: Vec<MaybePackage> = all_partials
        .into_iter()
        .map(|partial| match partial {
            MaybePackagePartial::Ok(partial) => {
                let install_set = install_set_by_path
                    .remove(partial.details.as_basic_data().ebuild_path.as_path())
                    .unwrap();
                let build_host_deps = build_host_deps_by_path
                    .remove(partial.details.as_basic_data().ebuild_path.as_path())
                    .unwrap();
                let bashrcs = config.package_bashrcs(&partial.details.as_thin_package_ref());

                MaybePackage::Ok(Arc::new(Package {
                    details: partial.details,
                    dependencies: partial.dependencies,
                    install_set,
                    sources: partial.sources,
                    build_host_deps,
                    bashrcs,
                }))
            }
            MaybePackagePartial::Err(err) => MaybePackage::Err(err),
        })
        .collect();

    packages
}
