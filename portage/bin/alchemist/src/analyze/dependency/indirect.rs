// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    cmp::Ordering,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Result};
use itertools::Itertools;

use crate::{
    analyze::{dependency::direct::DependencyKind, MaybePackageLocalAnalysis},
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
fn collect_transitive_dependencies(
    seed_packages: &[Arc<PackageDetails>],
    local_map: &HashMap<PathBuf, MaybePackageLocalAnalysis>,
    kinds: &[DependencyKind],
) -> Result<Vec<Arc<PackageDetails>>> {
    use std::collections::hash_map::Entry;

    let mut visited: HashMap<&Path, &Arc<PackageDetails>> = HashMap::new();
    let mut stack: Vec<&Arc<PackageDetails>> = seed_packages.iter().collect();

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

        let direct_dependencies = match local_map.get(ebuild_path).expect("local_map is exhaustive")
        {
            Ok(local) => &local.direct_dependencies,
            Err(error) => {
                bail!(
                    "Failed to analyze {}-{}: {}",
                    current.as_basic_data().package_name,
                    current.as_basic_data().version,
                    error.error
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

    let install_set = collect_transitive_dependencies(
        &[start_package.clone()],
        local_map,
        &[DependencyKind::RunTarget, DependencyKind::PostTarget],
    )?;

    let transitive_build_target_deps = collect_transitive_dependencies(
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

    Ok(IndirectDependencies {
        install_set,
        build_host_set,
    })
}
