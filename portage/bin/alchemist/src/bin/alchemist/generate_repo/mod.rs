// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub(self) mod common;
pub(self) mod deps;
pub mod internal;
pub(self) mod public;

use std::{
    collections::HashMap,
    fs::{create_dir_all, remove_dir_all, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use alchemist::{
    analyze::{
        dependency::{analyze_dependencies, PackageDependencies},
        source::{analyze_sources, PackageSources},
    },
    config::{bundle::ConfigBundle, ProvidedPackage},
    dependency::{package::PackageAtom, Predicate},
    ebuild::{CachedPackageLoader, MaybePackageDetails, PackageDetails},
    fakechroot::PathTranslator,
    repository::RepositorySet,
    resolver::PackageResolver,
};
use anyhow::{bail, Context, Result};
use itertools::{Either, Itertools};
use rayon::prelude::*;
use tracing::instrument;

use crate::alchemist::TargetData;

use self::{
    common::{MaybePackage, Package, PackageAnalysisError},
    deps::generate_deps_file,
    internal::overlays::generate_internal_overlays,
    internal::packages::{
        generate_internal_packages, PackageHostConfig, PackageTargetConfig, PackageType,
    },
    internal::{
        sdk::{
            generate_base_sdk, generate_host_sdk, generate_stage1_sdk, generate_target_sdk,
            SdkBaseConfig, SdkHostConfig, SdkTargetConfig,
        },
        sources::generate_internal_sources,
    },
    public::{generate_public_images, generate_public_packages},
};

#[instrument(skip_all)]
fn evaluate_all_packages(
    repos: &RepositorySet,
    loader: &CachedPackageLoader,
) -> Result<Vec<MaybePackageDetails>> {
    let ebuild_paths = repos.find_all_ebuilds()?;

    // Evaluate packages in parallel.
    let results = ebuild_paths
        .into_par_iter()
        .map(|ebuild_path| loader.load_package(&ebuild_path))
        .collect::<Result<Vec<_>>>()?;
    eprintln!("Loaded {} ebuilds", results.len());

    Ok(results)
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

/// Adds `current` and all of `current`'s runtime deps into to `runtime_deps`.
fn collect_runtime_deps<'a>(
    partial_by_path: &'a HashMap<&Path, &PackagePartial>,
    current: &'a Arc<PackageDetails>,
    runtime_deps: &mut HashMap<&'a Path, Arc<PackageDetails>>,
) {
    use std::collections::hash_map::Entry::*;
    match runtime_deps.entry(current.ebuild_path.as_path()) {
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
        .filter_map(|details| partial_by_path.get(details.ebuild_path.as_path()))
        .flat_map(|partial| &partial.dependencies.install_host_deps)
        .chain(&current.dependencies.build_host_deps)
        .sorted_by_key(|details| &details.ebuild_path)
        .unique_by(|details| &details.ebuild_path)
        .cloned()
        .collect()
}

#[instrument(skip_all)]
fn analyze_packages(
    config: &ConfigBundle,
    cross_compile: bool,
    all_details: Vec<MaybePackageDetails>,
    src_dir: &Path,
    host_resolver: Option<&PackageResolver>,
    target_resolver: &PackageResolver,
) -> Vec<MaybePackage> {
    // Analyze packages in parallel.
    let (all_partials, errors): (Vec<PackagePartial>, Vec<Arc<PackageAnalysisError>>) =
        all_details.into_par_iter().partition_map(|details| {
            let details = match details {
                MaybePackageDetails::Ok(details) => details,
                MaybePackageDetails::Err(error) => {
                    return Either::Right(Arc::new(PackageAnalysisError {
                        error: error.error.clone(),
                        details: MaybePackageDetails::Err(error),
                    }));
                }
            };
            let result = (|| -> Result<PackagePartial> {
                if details.masked {
                    // We do not support building masked packages because of
                    // edge cases: e.g., if one masked package depends on
                    // another masked one, this'd be treated as an unsatisfied
                    // dependency error.
                    bail!("The package is masked");
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
                Ok(package) => Either::Left(package),
                Err(err) => Either::Right(Arc::new(PackageAnalysisError {
                    details: MaybePackageDetails::Ok(details),
                    error: format!("{err:#}"),
                })),
            }
        });

    if !errors.is_empty() {
        eprintln!("WARNING: Analysis failed for {} packages", errors.len());
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

    let mut build_host_deps_by_path: HashMap<PathBuf, Vec<Arc<PackageDetails>>> = partial_by_path
        .iter()
        .map(|(path, partial)| {
            (
                path.to_path_buf(),
                compute_host_build_deps(&partial_by_path, partial),
            )
        })
        .collect();

    let ok_packages = all_partials.into_iter().map(|partial| {
        let install_set = install_set_by_path
            .remove(partial.details.ebuild_path.as_path())
            .unwrap();
        let build_host_deps = build_host_deps_by_path
            .remove(partial.details.ebuild_path.as_path())
            .unwrap();
        MaybePackage::Ok(Arc::new(Package {
            details: partial.details,
            dependencies: partial.dependencies,
            install_set,
            sources: partial.sources,
            build_host_deps,
        }))
    });
    let error_packages = errors.into_iter().map(MaybePackage::Err);

    ok_packages.chain(error_packages).collect()
}

fn load_packages(
    host: Option<&TargetData>,
    target: &TargetData,
    src_dir: &Path,
) -> Result<Vec<MaybePackage>> {
    eprintln!(
        "Loading packages for {}:{}...",
        target.board, target.profile
    );

    let details = evaluate_all_packages(&target.repos, &target.loader)?;

    eprintln!("Analyzing packages...");

    let cross_compile = if let Some(host) = host {
        let cbuild = host
            .config
            .env()
            .get("CHOST")
            .context("host is missing CHOST")?;
        let chost = target
            .config
            .env()
            .get("CHOST")
            .context("target is missing CHOST")?;
        cbuild != chost
    } else {
        true
    };

    let packages = analyze_packages(
        &target.config,
        cross_compile,
        details,
        src_dir,
        host.map(|x| &x.resolver),
        &target.resolver,
    );

    Ok(packages)
}

// Searches the `Package`s for the `atom` with the best version.
fn find_best_package_in(
    atom: &PackageAtom,
    packages: &[MaybePackage],
    resolver: &PackageResolver,
) -> Result<Option<Arc<Package>>> {
    // TODO(b/303400631): Consider `PackageAnalysisFailure`.
    let packages = packages
        .iter()
        .flat_map(|package| match package {
            MaybePackage::Ok(package) => Some(package),
            _ => None,
        })
        .collect_vec();

    let sdk_packages = packages
        .into_iter()
        .filter(|package| atom.matches(&package.details.as_thin_package_ref()))
        .collect_vec();

    let best_sdk_package_details = resolver.find_best_package_in(
        sdk_packages
            .iter()
            .map(|package| package.details.clone())
            .collect_vec()
            .as_slice(),
    )?;

    let best_sdk_package_details = match best_sdk_package_details {
        Some(best_sdk_package_details) => best_sdk_package_details,
        None => return Ok(None),
    };

    Ok(sdk_packages
        .into_iter()
        .find(|p| p.details.version == best_sdk_package_details.version)
        .cloned())
}

fn get_bootstrap_sdk_package(
    host_packages: &[MaybePackage],
    host_resolver: &PackageResolver,
) -> Result<Option<Arc<Package>>> {
    // TODO: Add a parameter to pass this along
    let sdk_atom = PackageAtom::from_str("virtual/target-chromium-os-sdk-bootstrap")?;

    find_best_package_in(&sdk_atom, host_packages, host_resolver)
}

/// Generates the stage1, stage2, etc packages and SDKs.
pub fn generate_stages(
    host: &TargetData,
    target: Option<&TargetData>,
    translator: &PathTranslator,
    src_dir: &Path,
    output_dir: &Path,
) -> Result<Vec<MaybePackage>> {
    let mut all_packages = vec![];

    let host_packages = load_packages(Some(host), host, src_dir)?;

    // When we install a set of packages into an SDK layer, any ebuilds that
    // use that SDK layer now have those packages provided for them, and they
    // no longer need to install them. Unfortunately we can't filter out these
    // "SDK layer packages" from the ebuild's dependency graph during bazel's
    // analysis phase because bazel doesn't like it when there are cycles in the
    // dependency graph. This means we need to filter out the dependencies
    // when we generate the BUILD files.
    let bootstrap_package = get_bootstrap_sdk_package(&host_packages, &host.resolver)?;
    let sdk_packages = bootstrap_package
        .as_ref()
        .map(|package| {
            package
                .install_set
                .iter()
                .map(|p| ProvidedPackage {
                    package_name: p.package_name.clone(),
                    version: p.version.clone(),
                })
                .collect_vec()
        })
        // TODO: Make this fail once all patches land
        .unwrap_or_else(Vec::new);

    // Generate the SDK used by the stage1/target/host packages.
    generate_stage1_sdk("stage1/target/host", host, translator, output_dir)?;

    // Generate the packages that will be built using the Stage 1 SDK.
    // These packages will be used to generate the Stage 2 SDK.
    //
    // i.e., An unknown version of LLVM will be used to cross-root build
    // the latest version of LLVM.
    //
    // We assume that the Stage 1 SDK contains all the BDEPENDs required
    // to build all the packages required to build the Stage 2 SDK.
    generate_internal_packages(
        // We don't know which packages are installed in the Stage 1 SDK
        // (downloaded SDK tarball), so we can't specify a host. In order
        // to build an SDK with known versions, we need to cross-root
        // compile a new SDK with the latest config and packages. This
        // guarantees that we can correctly track all the dependencies so
        // we can ensure proper package rebuilds.
        &PackageType::CrossRoot {
            host: None,
            target: PackageTargetConfig {
                board: &host.board,
                prefix: "stage1/target/host",
                repo_set: &host.repos,
            },
        },
        translator,
        // TODO: Do we want to pass in sdk_packages? This would mean we
        // can't manually build other host packages using the Stage 1 SDK,
        // but it saves us on generating symlinks for packages we probably
        // won't use.
        &host_packages,
        output_dir,
    )?;

    // Generate the stage 2 SDK
    //
    // This SDK will be used as the base for the host and target SDKs.
    // TODO: Make missing bootstrap_package fatal.
    if let Some(bootstrap_package) = bootstrap_package {
        generate_base_sdk(
            &SdkBaseConfig {
                name: "stage2",
                source_package_prefix: "stage1/target/host",
                // We use the `host:base` target because the stage1 SDK
                // `host` target lists all the primordial packages for the
                // target, and we don't want those pre-installed.
                source_sdk: "stage1/target/host:base",
                source_repo_set: &host.repos,
                bootstrap_package: bootstrap_package.as_ref(),
            },
            output_dir,
        )?;
    }

    // Generate the stage 2 host SDK. This will be used to build all the
    // host packages.
    generate_host_sdk(
        &SdkHostConfig {
            base: "stage2",
            name: "stage2/host",
            repo_set: &host.repos,
            profile: &host.profile,
        },
        output_dir,
    )?;

    // Generate the host packages that will be built using the Stage 2 SDK.
    let stage2_host = PackageHostConfig {
        repo_set: &host.repos,
        prefix: "stage2/host",
        sdk_provided_packages: &sdk_packages,
    };
    generate_internal_packages(
        // We no longer need to cross-root build since we know exactly what
        // is contained in the Stage 2 SDK. This means we can properly
        // support BDEPENDs. All the packages listed in `sdk_packages` are
        // considered implicit system dependencies for any of these
        // packages.
        &PackageType::Host(stage2_host),
        translator,
        &host_packages,
        output_dir,
    )?;

    // Generate the Stage 3 host SDK
    //
    // The stage 2 SDK is composed of packages built using the Stage 1 SDK.
    // The stage 3 SDK will be composed of packages built using the Stage 2
    // SDK. This means we can verify that the latest toolchain can bootstrap
    // itself.
    // i.e., Latest LLVM can build Latest LLVM.
    // TODO: Add call to generate stage3 sdk
    // TODO: Also support building a "bootstrap" SDK target that is composed
    // of ALL BDEPEND + RDEPEND + DEPEND of the
    // virtual/target-chromium-os-sdk-bootstrap package.

    // TODO: Add stage3/host package if we decide we want to build targets
    // against the stage 3 SDK.

    generate_public_packages(
        &host_packages,
        &host.resolver,
        &[
            public::TargetConfig {
                config: "stage1",
                prefix: "stage1/target/host",
            },
            public::TargetConfig {
                config: "stage2",
                prefix: "stage2/host",
            },
        ],
        "stage2/host",
        &output_dir.join("host"),
    )?;

    all_packages.extend(host_packages);

    if let Some(target) = target {
        let target_packages = load_packages(Some(host), target, src_dir)?;

        generate_stage1_sdk("stage1/target/board", target, translator, output_dir)?;

        generate_internal_packages(
            // The same comment applies here as the stage1/target/host packages.
            // We don't know what packages are installed in the Stage 1 SDK,
            // so we can't support BDEPENDs.
            &PackageType::CrossRoot {
                host: None,
                target: PackageTargetConfig {
                    board: &target.board,
                    prefix: "stage1/target/board",
                    repo_set: &target.repos,
                },
            },
            translator,
            &target_packages,
            output_dir,
        )?;

        // Generate the stage 2 target board SDK. This will be used to build
        // all the target's packages.
        generate_target_sdk(
            &SdkTargetConfig {
                base: "stage2",
                host_prefix: "stage2/host",
                host_resolver: &host.resolver,
                name: "stage2/target/board",
                board: &target.board,
                target_repo_set: &target.repos,
                target_resolver: &target.resolver,
                target_primary_toolchain: Some(
                    target
                        .toolchains
                        .primary()
                        .context("Target is missing primary toolchain")?,
                ),
            },
            output_dir,
        )?;

        // Generate the target packages that will be cross-root /
        // cross-compiled using the Stage 2 SDK.
        generate_internal_packages(
            &PackageType::CrossRoot {
                // We want to use the stage2/host packages to satisfy
                // our BDEPEND/IDEPEND dependencies.
                host: Some(stage2_host),
                target: PackageTargetConfig {
                    board: &target.board,
                    prefix: "stage2/target/board",
                    repo_set: &target.repos,
                },
            },
            translator,
            &target_packages,
            output_dir,
        )?;

        // TODO(b/303136802): Delete this once we migrate the CQ builders.
        generate_public_packages(
            &target_packages,
            &target.resolver,
            &[
                public::TargetConfig {
                    config: "stage1",
                    prefix: "stage1/target/board",
                },
                public::TargetConfig {
                    config: "stage2",
                    prefix: "stage2/target/board",
                },
            ],
            "stage1/target/board",
            output_dir,
        )?;

        generate_public_packages(
            &target_packages,
            &target.resolver,
            &[
                public::TargetConfig {
                    config: "stage1",
                    prefix: "stage1/target/board",
                },
                public::TargetConfig {
                    config: "stage2",
                    prefix: "stage2/target/board",
                },
            ],
            "stage1/target/board",
            &output_dir.join("target"),
        )?;

        generate_public_images(&target.board, &output_dir.join("images"))?;

        // TODO: Generate the Stage 3 target packages if we decide to build
        // targets against the stage 3 SDK.

        all_packages.extend(target_packages);
    }

    Ok(all_packages)
}

/// The entry point of "generate-repo" subcommand.
pub fn generate_repo_main(
    host: &TargetData,
    target: Option<&TargetData>,
    translator: &PathTranslator,
    src_dir: &Path,
    output_dir: &Path,
    deps_file: &Path,
) -> Result<()> {
    match remove_dir_all(output_dir) {
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        err => {
            err?;
        }
    };
    create_dir_all(output_dir)?;

    let _guard = cliutil::LoggingConfig {
        trace_file: Some(output_dir.join("trace.json")),
        log_file: None,
        console_logger: None,
    }
    .setup()?;

    eprintln!("Generating @portage...");

    generate_internal_overlays(
        translator,
        [Some(host), target]
            .iter()
            .filter_map(|x| x.map(|data| data.repos.as_ref()))
            .collect_vec()
            .as_slice(),
        output_dir,
    )?;

    let all_packages = generate_stages(host, target, translator, src_dir, output_dir)?;

    generate_deps_file(
        &all_packages
            .iter()
            .flat_map(|package| match package {
                MaybePackage::Ok(package) => Some(package.as_ref()),
                _ => None,
            })
            .collect_vec(),
        deps_file,
    )?;

    File::create(output_dir.join("BUILD.bazel"))?
        .write_all(include_bytes!("templates/root.BUILD.bazel"))?;
    File::create(output_dir.join("WORKSPACE.bazel"))?.write_all(&[])?;

    eprintln!("Generating sources...");
    generate_internal_sources(
        all_packages.iter().flat_map(|package| match package {
            MaybePackage::Ok(package) => package.sources.local_sources.as_slice(),
            _ => &[],
        }),
        src_dir
            .parent()
            .expect("src_dir '{src_dir:?} to have a parent"),
        output_dir,
    )?;

    eprintln!("Generated @portage.");
    Ok(())
}
