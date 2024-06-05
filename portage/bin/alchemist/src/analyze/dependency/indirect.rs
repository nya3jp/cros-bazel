// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Result};
use itertools::Itertools;

use crate::{
    analyze::{
        dependency::direct::{DependencyKind, DirectDependencies},
        MaybePackageLocalAnalysis, PackageAnalysisError, PackageLocalAnalysis,
    },
    ebuild::PackageDetails,
};

pub struct IndirectDependencies {
    /// A list of packages needed to install together with this package.
    /// Specifically, it is a transitive closure of dependencies introduced by
    /// RDEPEND and PDEPEND. Alchemist needs to compute it, instead of letting
    /// Bazel compute it, because there can be circular dependencies.
    pub install_set: Vec<Arc<PackageDetails>>,

    /// A list of host packages needed to install when building this package.
    /// Namely, it includes the BDEPENDs declared by this package and all the
    /// IDEPENDs specified by the package's DEPENDs and their transitive
    /// RDEPENDs.
    ///
    /// When building the ephemeral CrOS SDK for building the package, we need
    /// to ensure that all the IDEPENDs are installed. We could add the concept
    /// of an IDEPEND to Bazel, but it would make the `sdk_install_deps` rule
    /// very complicated and harder to understand.
    ///
    /// This list does NOT necessarily include all host packages to install, and
    /// may omit some transitive runtime dependencies represented by RDEPEND and
    /// IDEPEND. Bazel must make sure to install those transitive dependencies
    /// as well on setting up an ephemeral CrOS SDK for building the package.
    /// Alchemist doesn't compute the full transitive closure while it's
    /// technically possible because it unnecessarily complicates the dependency
    /// calculation logic.
    pub build_host_set: Vec<Arc<PackageDetails>>,

    /// Host packages in this package's set of reusable dependencies.
    ///
    /// The "reusable dependencies" of a package P are the subset of P's
    /// dependencies which are also required by any other packages that DEPEND
    /// on P. We define the set of reusable dependencies of P as the subset of
    /// P's direct dependencies that are both DEPEND and RDEPEND dependencies,
    /// plus any other dependencies that can be reached directly or
    /// transitively from the former via RDEPEND and IDEPEND edges in the
    /// dependency graph.
    ///
    /// When assembling the ephemeral CrOS SDK to build a package P, we first
    /// install P's reusable dependencies into a "reusable SDK" based on a
    /// generic SDK. Then, we install P's exclusive dependencies (i.e. all
    /// other non-reusable dependencies) into an "exclusive SDK" based on the
    /// reusable SDK. The exclusive SDK has all the required packages to build
    /// P.
    ///
    /// We leverage the notion of reusable dependencies to speed up the
    /// construction of SDKs when extending a given base SDK with a set of
    /// additional packages, which we refer to as the "install set". Rather than
    /// installing those packages into the given base SDK, we first look for
    /// better candidate base SDKs among the reusable SDKs of the packages in
    /// the install set. A reusable SDK is considered a candidate iff the set
    /// of packages it contains is a subset of the install set. The best
    /// candidate base SDK is the one which contains the largest number of
    /// packages in the install set. Once the best candidate base SDK is
    /// identified, we extend it by installing into it any packages in the
    /// install set that are not already present in it. In other words, we
    /// reuse the dependency installations of the package associated with the
    /// best candidate base SDK. This technique can save hundreds of dependency
    /// installations, e.g. for some packages high in the dependency graph.
    pub reusable_host_set: Vec<Arc<PackageDetails>>,

    /// Target packages in this package's set of reusable dependencies.
    pub reusable_target_set: Vec<Arc<PackageDetails>>,
}

fn compare_packages(a: &&Arc<PackageDetails>, b: &&Arc<PackageDetails>) -> Ordering {
    let a = a.as_basic_data();
    let b = b.as_basic_data();
    a.package_name
        .cmp(&b.package_name)
        .then(a.version.cmp(&b.version))
}

/// Collects transitive dependencies of the given seed packages.
///
/// Seed packages specify the set of packages to start searching from. Seed
/// packages are always included in the result package set.
///
/// `kinds` specifies dependency kinds to follow.
pub fn collect_transitive_dependencies<'a, D, DRef, P, E, R>(
    seed_packages: impl IntoIterator<Item = &'a Arc<PackageDetails>>,
    local_map: &'a HashMap<P, R>,
    kinds: &'a [DependencyKind],
) -> Result<Vec<Arc<PackageDetails>>>
where
    D: AsRef<DirectDependencies>,
    DRef: Borrow<D>,
    P: Borrow<Path> + std::cmp::Eq + std::hash::Hash,
    E: Borrow<PackageAnalysisError>,
    R: Borrow<Result<DRef, E>>,
{
    use std::collections::hash_map::Entry;

    let mut visited: HashMap<&Path, &Arc<PackageDetails>> = HashMap::new();
    let mut stack: Vec<&Arc<PackageDetails>> = seed_packages.into_iter().collect();

    // Search the dependency graph with DFS.
    while let Some(current) = stack.pop() {
        let ebuild_path = current.as_basic_data().ebuild_path.as_path();

        // Skip already-visited packages.
        match visited.entry(ebuild_path) {
            Entry::Occupied(_) => continue,
            Entry::Vacant(entry) => {
                entry.insert(current);
            }
        }

        let maybe_package = local_map
            .get(ebuild_path)
            .expect("local_map is exhaustive")
            .borrow();

        let direct_dependencies = match maybe_package {
            Ok(local) => local.borrow().as_ref(),
            Err(error) => {
                bail!(
                    "Failed to analyze {}-{}: {}",
                    current.as_basic_data().package_name,
                    current.as_basic_data().version,
                    error.borrow().error
                );
            }
        };

        for kind in kinds {
            stack.extend(direct_dependencies.get(*kind));
        }
    }

    let packages = visited
        .into_values()
        .sorted_by(compare_packages)
        .cloned()
        .collect();
    Ok(packages)
}

/// Collects direct install-time host dependencies (IDEPEND) of the given
/// packages.
fn collect_direct_host_install_dependencies(
    packages: &[Arc<PackageDetails>],
    local_map: &HashMap<PathBuf, MaybePackageLocalAnalysis>,
) -> Result<Vec<Arc<PackageDetails>>> {
    let mut visited: HashMap<&Path, &Arc<PackageDetails>> = HashMap::new();

    for package in packages {
        let ebuild_path = package.as_basic_data().ebuild_path.as_path();

        let direct_dependencies = match local_map.get(ebuild_path).expect("local_map is exhaustive")
        {
            Ok(local) => &local.direct_dependencies,
            Err(error) => {
                bail!(
                    "Failed to analyze {}-{}: {}",
                    package.as_basic_data().package_name,
                    package.as_basic_data().version,
                    error.error
                );
            }
        };

        for p in &direct_dependencies.install_host {
            visited.insert(ebuild_path, p);
        }
    }

    let packages = visited
        .into_values()
        .sorted_by(compare_packages)
        .cloned()
        .collect();
    Ok(packages)
}

/// Returns the direct dependencies of the given package.
fn get_direct_deps<'a>(
    package: &Arc<PackageDetails>,
    local_map: &'a HashMap<PathBuf, MaybePackageLocalAnalysis>,
) -> Result<&'a DirectDependencies> {
    return match local_map
        .get(package.as_basic_data().ebuild_path.as_path())
        .expect("local_map is exhaustive")
    {
        Ok(local) => Ok(&local.direct_dependencies),
        Err(error) => {
            bail!(
                "Failed to analyze {}-{}: {}",
                package.as_basic_data().package_name,
                package.as_basic_data().version,
                error.error
            );
        }
    };
}

/// Computes the intersection of two lists of PackageDetails.
fn package_intersection<'a>(
    a: &'a Vec<Arc<PackageDetails>>,
    b: &'a Vec<Arc<PackageDetails>>,
) -> Vec<&'a Arc<PackageDetails>> {
    let b_ebuild_paths: HashSet<PathBuf> = b
        .into_iter()
        .map(|package| package.as_basic_data().ebuild_path.clone())
        .collect();
    a.into_iter()
        .filter(|package| b_ebuild_paths.contains(&package.as_basic_data().ebuild_path))
        .collect_vec()
}

/// Collects the given package's "reusable dependencies", that is, the subset
/// of the package's dependencies which are required by all other packages that
/// DEPEND on it.
///
/// This function returns two vectors: one with the package's reusable host
/// dependencies and another with the package's reusable target dependencies.
fn collect_reusable_dependencies(
    package: &Arc<PackageDetails>,
    local_map: &HashMap<PathBuf, MaybePackageLocalAnalysis>,
) -> Result<(Vec<Arc<PackageDetails>>, Vec<Arc<PackageDetails>>)> {
    use std::collections::hash_map::Entry;

    #[derive(Clone, Copy)]
    enum PackageType {
        Host,
        Target,
    }

    let direct_dependencies = get_direct_deps(package, local_map)?;

    // Populate the DFS stack with the intersection of the direct DEPEND and RDEPEND dependencies.
    let mut stack: Vec<(&Arc<PackageDetails>, PackageType)> = package_intersection(
        direct_dependencies.get(DependencyKind::BuildTarget),
        direct_dependencies.get(DependencyKind::RunTarget),
    )
    .into_iter()
    .map(|package| (package, PackageType::Target))
    .collect();

    let mut visited_host = HashMap::new();
    let mut visited_target = HashMap::new();

    // Search the dependency graph with DFS along RDEPEND and IDEPEND edges.
    while let Some((current, package_type)) = stack.pop() {
        let ebuild_path = current.as_basic_data().ebuild_path.as_path();

        // Mark package as visited, or skip if already visited.
        match package_type {
            PackageType::Host => match visited_host.entry(ebuild_path) {
                Entry::Occupied(_) => continue,
                _ => visited_host.insert(ebuild_path, current),
            },
            PackageType::Target => match visited_target.entry(ebuild_path) {
                Entry::Occupied(_) => continue,
                _ => visited_target.insert(ebuild_path, current),
            },
        };

        let direct_dependencies = get_direct_deps(current, local_map)?;
        stack.extend(
            direct_dependencies
                .run_target
                .iter()
                .map(|package| (package, package_type)),
        );
        stack.extend(
            direct_dependencies
                .install_host
                .iter()
                .map(|package| (package, PackageType::Host)),
        );
    }

    let host_reusable_dependencies = visited_host
        .into_values()
        .sorted_by(compare_packages)
        .cloned()
        .collect();

    let target_reusable_dependencies = visited_target
        .into_values()
        .sorted_by(compare_packages)
        .cloned()
        .collect();

    Ok((host_reusable_dependencies, target_reusable_dependencies))
}

pub fn analyze_indirect_dependencies(
    start_package: &Arc<PackageDetails>,
    local_map: &HashMap<PathBuf, MaybePackageLocalAnalysis>,
) -> Result<IndirectDependencies> {
    let local = match local_map
        .get(start_package.as_basic_data().ebuild_path.as_path())
        .expect("local_map is exhaustive")
    {
        Ok(local) => local,
        Err(error) => bail!("{}", error.error),
    };

    let install_set = collect_transitive_dependencies::<PackageLocalAnalysis, _, _, _, _>(
        [start_package],
        local_map,
        &[DependencyKind::RunTarget, DependencyKind::PostTarget],
    )?;

    let transitive_build_target_deps =
        collect_transitive_dependencies::<PackageLocalAnalysis, _, _, _, _>(
            &local.direct_dependencies.build_target,
            local_map,
            // No need to follow PDEPEND on deciding build-time dependencies.
            &[DependencyKind::RunTarget],
        )?;

    let mut build_host_set = local.direct_dependencies.build_host.clone();
    build_host_set.extend(collect_direct_host_install_dependencies(
        &transitive_build_target_deps,
        local_map,
    )?);

    let (reusable_host_set, reusable_target_set) =
        collect_reusable_dependencies(start_package, local_map)?;

    Ok(IndirectDependencies {
        install_set,
        build_host_set,
        reusable_host_set,
        reusable_target_set,
    })
}
