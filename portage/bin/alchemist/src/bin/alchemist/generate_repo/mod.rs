// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod common;
mod deps;
pub mod internal;
mod public;

use std::{
    collections::HashMap,
    fs::{create_dir_all, remove_dir_all, File},
    io::{ErrorKind, Write},
    path::Path,
    str::FromStr,
    sync::Arc,
};

use alchemist::{
    analyze::{
        analyze_packages, dependency::direct::DependencyKind,
        dependency::indirect::collect_transitive_dependencies, MaybePackage, Package,
        PackageAnalysisError,
    },
    config::ProvidedPackage,
    dependency::package::{AsPackageRef, PackageAtom},
    ebuild::PackageDetails,
    fakechroot::PathTranslator,
    resolver::select_best_version,
};
use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;

use crate::alchemist::TargetData;

use self::{
    deps::generate_deps_file,
    internal::bashrcs::generate_internal_bashrcs,
    internal::overlays::generate_internal_overlays,
    internal::packages::{
        generate_internal_packages, PackageHostConfig, PackageTargetConfig, PackageType,
    },
    internal::{
        portage_config::generate_portage_config,
        sdk::{
            generate_base_sdk, generate_host_sdk, generate_stage1_sdk, generate_target_sdk,
            SdkBaseConfig, SdkHostConfig, SdkTargetConfig,
        },
        sources::generate_internal_sources,
    },
    public::{generate_public_images, generate_public_packages},
};

fn load_packages(
    host: &TargetData,
    target: &TargetData,
    src_dir: &Path,
) -> Result<Vec<MaybePackage>> {
    eprintln!(
        "Loading packages for {}:{}...",
        target.board, target.profile
    );

    let cross_compile = {
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
    };

    let packages = analyze_packages(
        &target.config,
        cross_compile,
        src_dir,
        &host.resolver,
        &target.resolver,
    )?;

    eprintln!("Loaded {} packages", packages.len());

    Ok(packages)
}

fn get_sdk_implicit_system_package(host_packages: &[MaybePackage]) -> Result<Arc<Package>> {
    // TODO: Add a parameter to pass this along
    let sdk_atom = PackageAtom::from_str("virtual/target-sdk-implicit-system")?;

    let best_package = select_best_version(
        host_packages
            .iter()
            .filter(|package| sdk_atom.matches(&package.as_package_ref())),
    )
    .with_context(|| format!("Could not find {sdk_atom}"))?;

    match best_package {
        MaybePackage::Ok(package) => Ok(package.clone()),
        MaybePackage::Err(err) => bail!(
            "Cannot determine the best version for {}: {}-{}: {}",
            sdk_atom,
            err.as_basic_data().package_name,
            err.as_basic_data().version,
            err.error
        ),
    }
}

fn compute_provided_packages(
    packages_by_path: &HashMap<&Path, Result<&Package, &PackageAnalysisError>>,
    root: &Package,
) -> Result<Vec<ProvidedPackage>> {
    Ok(
        collect_transitive_dependencies::<alchemist::analyze::Package, _, _, _, _>(
            [&root.details],
            packages_by_path,
            &[DependencyKind::RunTarget],
        )?
        .into_iter()
        .map(|package| ProvidedPackage {
            package_name: package.as_basic_data().package_name.clone(),
            version: package.as_basic_data().version.clone(),
        })
        .sorted()
        .collect(),
    )
}

/// The bootstrap packages are all the BDEPENDs required to build the transitive
/// DEPEND and RDEPEND of the `root` package.
fn compute_bootstrap_packages<'a>(
    packages_by_path: &'a HashMap<&Path, Result<&Package, &PackageAnalysisError>>,
    root: &Package,
) -> Result<Vec<&'a Package>> {
    // We collect the DEPEND in addition to the RDEPEND because there might be
    // packages that only declare a dependency as a DEPEND. If we only collected
    // the RDEPEND then we might not be able to build the DEPEND and thus fail
    // to build the implicit system set.
    let depend_and_rdepend =
        collect_transitive_dependencies::<alchemist::analyze::Package, _, _, _, _>(
            [&root.details],
            packages_by_path,
            &[DependencyKind::BuildTarget, DependencyKind::RunTarget],
        )?;

    let mut bdepends = vec![];

    let get_package = |details: &PackageDetails| -> Result<_> {
        let maybe_package = packages_by_path
            .get(details.as_basic_data().ebuild_path.as_path())
            .context("package doesn't to exist")?;

        maybe_package.map_err(|err| {
            anyhow!(
                "{} failed to analyze: {}",
                err.as_basic_data().package_name,
                err.error
            )
        })
    };

    // Get all the direct BDEPEND and IDEPEND for the DEPEND/RDEPENDs.
    // We don't traverse the RDEPEND of the BDEPEND because we rely on bazel
    // to compute the transitive dependencies of the BDEPEND.
    for details in depend_and_rdepend {
        let package = get_package(&details)?;
        for dep in package
            .dependencies
            .direct
            .build_host
            .iter()
            .chain(&package.dependencies.direct.install_host)
        {
            bdepends.push(get_package(dep)?);
        }
    }

    Ok(bdepends
        .into_iter()
        .unique_by(|package| &package.as_basic_data().ebuild_path)
        .sorted_by_key(|package| &package.as_basic_data().ebuild_path)
        .collect())
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

    let host_packages = load_packages(host, host, src_dir)?;

    let packages_by_path = host_packages
        .iter()
        .map(|package| {
            (
                package.as_basic_data().ebuild_path.as_path(),
                package.into(),
            )
        })
        .collect();

    // When we install a set of packages into an SDK layer, any ebuilds that
    // use that SDK layer now have those packages provided for them, and they
    // no longer need to install them. Unfortunately we can't filter out these
    // "SDK layer packages" from the ebuild's dependency graph during bazel's
    // analysis phase because bazel doesn't like it when there are cycles in the
    // dependency graph. This means we need to filter out the dependencies
    // when we generate the BUILD files.
    let implicit_system_package = get_sdk_implicit_system_package(&host_packages)?;
    let implicit_system_packages =
        compute_provided_packages(&packages_by_path, &implicit_system_package)?;

    // Generate the SDK used by the stage1/target/host packages.
    generate_stage1_sdk("stage1/target/host", host, output_dir)?;

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
    generate_base_sdk(
        &SdkBaseConfig {
            name: "stage2",
            source_package_prefix: "stage1/target/host",
            // We use the `host:base` target because the stage1 SDK
            // `host` target lists all the primordial packages for the
            // target, and we don't want those pre-installed.
            source_sdk: "stage1/target/host:base",
            source_repo_set: &host.repos,
            packages: vec![&implicit_system_package],
            package_suffix: None,
        },
        output_dir,
    )?;

    // Generate the stage 2 host SDK. This will be used to build all the
    // host packages.
    generate_host_sdk(
        &SdkHostConfig {
            base: "stage2",
            name: "stage2/host",
        },
        output_dir,
    )?;

    // Generate the host packages that will be built using the Stage 2 SDK.
    let stage2_host = PackageHostConfig {
        repo_set: &host.repos,
        prefix: "stage2/host",
        sdk_provided_packages: &implicit_system_packages,
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

    // Generate the Stage 3 Bootstrap SDK
    //
    // The stage 3 Bootstrap SDK is composed of packages built using the Stage
    // 2 SDK. It is used to update the Stage 1 SDK when necessary. It only
    // contains the packages necessary to build the implicit system set.
    generate_base_sdk(
        &SdkBaseConfig {
            name: "stage3:bootstrap",
            source_package_prefix: "stage2/host",
            // TODO: THIS IS WRONG, we should be using the stage2 SDK, but
            // we don't have a target/host SDK right now.
            source_sdk: "stage1/target/host:base",
            source_repo_set: &host.repos,
            packages: std::iter::once(implicit_system_package.as_ref())
                .chain(compute_bootstrap_packages(
                    &packages_by_path,
                    &implicit_system_package,
                )?)
                .collect(),
            // We use the _including_provided suffix so we can get ALL the
            // RDEPENDs. The regular ebuild target has its RDEPENDs filtered
            // by what the SDK already provides.
            package_suffix: Some("_including_provided"),
        },
        output_dir,
    )?;

    // Generate the SDK used by the stage3/target/host packages.
    generate_target_sdk(
        &SdkTargetConfig {
            base: "stage3:bootstrap",
            host_prefix: "TODO: REMOVE ME. Only used then target_primary_toolchain is set",
            host_resolver: &host.resolver,
            name: "stage3/target/host",
            board: &host.board,
            target_repo_set: &host.repos,
            target_resolver: &host.resolver,
            target_primary_toolchain: None,
        },
        output_dir,
    )?;

    // These packages will be used to test that the stage 3 bootstrap SDK can
    // correctly build the implicit system set.
    generate_internal_packages(
        &PackageType::CrossRoot {
            host: None,
            target: PackageTargetConfig {
                board: &host.board,
                prefix: "stage3/target/host",
                repo_set: &host.repos,
            },
        },
        translator,
        // TODO: Do we want to pass in only the DEPEND + RDEPEND of the implicit
        // system?
        &host_packages,
        output_dir,
    )?;

    // Generate the stage 4 SDK
    //
    // This SDK is only used to verify that the Stage 3 Bootstrap SDK can
    // actually bootstrap the implicit system.
    //
    // The Stage 2 SDK and Stage 4 SDK should in theory be bit-for-bit
    // identical.
    generate_base_sdk(
        &SdkBaseConfig {
            name: "stage4",
            source_package_prefix: "stage3/target/host",
            // We use the `host:base` target because the stage3 SDK
            // `host` target lists all the primordial packages for the
            // target, and we don't want those pre-installed.
            source_sdk: "stage3/target/host:base",
            source_repo_set: &host.repos,
            packages: vec![&implicit_system_package],
            package_suffix: None,
        },
        output_dir,
    )?;

    // Generate public aliases
    generate_public_packages(
        &host_packages,
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
        let target_packages = load_packages(host, target, src_dir)?;

        generate_stage1_sdk("stage1/target/board", target, output_dir)?;

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

        generate_public_packages(
            &target_packages,
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

    generate_internal_bashrcs(translator, host, target, output_dir)?;

    let all_packages = generate_stages(host, target, translator, src_dir, output_dir)?;

    generate_deps_file(
        &all_packages
            .iter()
            .flat_map(|package| match package {
                MaybePackage::Ok(package) => Some(&package.sources),
                _ => None,
            })
            .collect_vec(),
        deps_file,
    )?;

    generate_portage_config(host, target, output_dir)?;

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
