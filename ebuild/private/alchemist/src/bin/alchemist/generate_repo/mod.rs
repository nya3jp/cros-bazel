// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub(self) mod common;
pub mod internal;
pub(self) mod public;
pub(self) mod repositories;
pub(self) mod settings;

use std::{
    collections::HashMap,
    fs::{create_dir_all, remove_dir_all, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use alchemist::{
    analyze::{
        dependency::{analyze_dependencies, PackageDependencies},
        source::{analyze_sources, PackageSources},
    },
    config::bundle::ConfigBundle,
    ebuild::{CachedPackageLoader, PackageDetails, Stability},
    fakechroot::PathTranslator,
    repository::RepositorySet,
    resolver::PackageResolver,
};
use anyhow::Result;
use itertools::{Either, Itertools};
use rayon::prelude::*;

use crate::alchemist::TargetData;

use self::{
    common::{AnalysisError, Package},
    internal::overlays::generate_internal_overlays,
    internal::packages::generate_internal_packages,
    internal::{sdk::generate_sdk, sources::generate_internal_sources},
    public::generate_public_packages,
    repositories::generate_repositories_file,
    settings::generate_settings_bzl,
};

fn evaluate_all_packages(
    repos: &RepositorySet,
    loader: &CachedPackageLoader,
) -> Result<Vec<Arc<PackageDetails>>> {
    let ebuild_paths = repos.find_all_ebuilds()?;
    let ebuild_count = ebuild_paths.len();

    let counter = Arc::new(AtomicUsize::new(0));

    // Evaluate packages in parallel.
    let packages = ebuild_paths
        .into_par_iter()
        .map(|ebuild_path| {
            let result = loader.load_package(&ebuild_path);
            let count = 1 + counter.fetch_add(1, Ordering::SeqCst);
            eprint!("Loading ebuilds... {}/{}\r", count, ebuild_count);
            result
        })
        .collect::<Result<Vec<_>>>()?;
    eprintln!();

    Ok(packages)
}

/// Similar to [`Package`], but an install set is not resolved yet.
struct PackagePartial {
    pub details: Arc<PackageDetails>,
    pub dependencies: PackageDependencies,
    pub sources: PackageSources,
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
    match install_map.entry(current.ebuild_path.as_path()) {
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
    let current_partial = match partial_by_path.get(current.ebuild_path.as_path()) {
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

fn analyze_packages(
    config: &ConfigBundle,
    all_details: Vec<Arc<PackageDetails>>,
    src_dir: &Path,
    resolver: &PackageResolver,
) -> (Vec<Package>, Vec<AnalysisError>) {
    // Analyze packages in parallel.
    let (all_partials, failures): (Vec<PackagePartial>, Vec<AnalysisError>) =
        all_details.par_iter().partition_map(|details| {
            let result = (|| -> Result<PackagePartial> {
                let dependencies = analyze_dependencies(details, resolver)?;
                let sources = analyze_sources(config, details, src_dir)?;
                Ok(PackagePartial {
                    details: details.clone(),
                    dependencies,
                    sources,
                })
            })();
            match result {
                Ok(package) => Either::Left(package),
                Err(err) => Either::Right(AnalysisError {
                    repo_name: details.repo_name.clone(),
                    package_name: details.package_name.clone(),
                    ebuild: details.ebuild_path.clone(),
                    version: details.version.clone(),
                    error: format!("{err:#}"),
                }),
            }
        });

    if !failures.is_empty() {
        eprintln!("WARNING: Analysis failed for {} packages", failures.len());
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
        .map(|partial| (partial.details.ebuild_path.as_path(), partial))
        .collect();

    let mut install_set_by_path: HashMap<PathBuf, Vec<Arc<PackageDetails>>> = partial_by_path
        .iter()
        .map(|(path, partial)| {
            let mut install_map: HashMap<&Path, Arc<PackageDetails>> = HashMap::new();
            find_install_map(&partial_by_path, &partial.details, &mut install_map);

            let install_set = install_map
                .into_values()
                .sorted_by(|a, b| {
                    a.package_name
                        .cmp(&b.package_name)
                        .then_with(|| a.version.cmp(&b.version))
                })
                .collect();

            ((*path).to_owned(), install_set)
        })
        .collect();

    let packages = all_partials
        .into_iter()
        .map(|partial| {
            let install_set = install_set_by_path
                .remove(partial.details.ebuild_path.as_path())
                .unwrap();
            Package {
                details: partial.details,
                dependencies: partial.dependencies,
                install_set,
                sources: partial.sources,
            }
        })
        .collect();

    (packages, failures)
}

fn load_packages(
    target: &TargetData,
    src_dir: &Path,
) -> Result<(Vec<Package>, Vec<AnalysisError>)> {
    eprintln!(
        "Loading packages for {}:{}...",
        target.board, target.profile
    );

    let mut all_details = evaluate_all_packages(&target.repos, &target.loader)?;

    // We don't want to generate targets for packages that are marked broken
    // for the arch. i.e., a x86 only package shouldn't be visible for an arm64
    // build.
    all_details.retain(|package| package.stability != Stability::Broken);

    eprintln!("Analyzing packages...");

    Ok(analyze_packages(
        &target.config,
        all_details,
        src_dir,
        &target.resolver,
    ))
}
/// The entry point of "generate-repo" subcommand.
pub fn generate_repo_main(
    target: TargetData,
    translator: &PathTranslator,
    src_dir: &Path,
    output_dir: &Path,
) -> Result<()> {
    match remove_dir_all(output_dir) {
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        err => {
            err?;
        }
    };
    create_dir_all(output_dir)?;

    let (target_packages, target_failures) = load_packages(&target, src_dir)?;

    let all_local_sources = target_packages
        .iter()
        .flat_map(|package| &package.sources.local_sources);

    eprintln!("Generating @portage...");

    generate_internal_overlays(translator, &target.repos, output_dir)?;
    generate_internal_packages(translator, &target_packages, &target_failures, output_dir)?;
    generate_internal_sources(all_local_sources, src_dir, output_dir)?;
    generate_public_packages(&target_packages, &target.resolver, output_dir)?;
    generate_repositories_file(&target_packages, &output_dir.join("repositories.bzl"))?;
    generate_settings_bzl(&target.board, &output_dir.join("settings.bzl"))?;
    generate_sdk(
        &target.board,
        &target.profile,
        &target.repos,
        &target.toolchains,
        translator,
        output_dir,
    )?;

    File::create(output_dir.join("BUILD.bazel"))?
        .write_all(include_bytes!("templates/root.BUILD.bazel"))?;
    File::create(output_dir.join("WORKSPACE.bazel"))?.write_all(&[])?;

    eprintln!("Generated @portage.");

    Ok(())
}
