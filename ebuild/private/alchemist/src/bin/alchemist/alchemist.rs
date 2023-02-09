// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::os::unix::fs;

use std::path::Path;
use std::{env::current_dir, path::PathBuf};

use crate::digest_repo::digest_repo_main;
use crate::dump_deps::dump_deps_main;
use crate::dump_package::dump_package_main;
use crate::generate_repo::generate_repo_main;

use alchemist::config::makeconf::generate::generate_make_conf_for_board;
use alchemist::fakechroot::PathTranslator;
use alchemist::fileops::{execute_file_ops, FileOps};
use alchemist::repository::RepositoryLookup;
use alchemist::toolchain::ToolchainConfig;
use alchemist::{
    config::{
        bundle::ConfigBundle, profile::Profile, site::SiteSettings, ConfigNode, ConfigNodeValue,
        ConfigSource, PackageMaskKind, PackageMaskUpdate, ProvidedPackage, SimpleConfigSource,
    },
    dependency::package::PackageAtomDependency,
    ebuild::{CachedPackageLoader, PackageLoader},
    fakechroot::enter_fake_chroot,
    repository::RepositorySet,
    resolver::PackageResolver,
    toolchain::load_toolchains,
};
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use tempfile::TempDir;

#[derive(Parser, Debug)]
#[command(name = "alchemist")]
#[command(author = "ChromiumOS Authors")]
#[command(about = "Analyzes Portage trees", long_about = None)]
pub struct Args {
    /// Board name to build packages for.
    #[arg(short = 'b', long, value_name = "NAME")]
    board: String,

    /// Path to the ChromiumOS source directory root.
    /// If unset, it is inferred from the current directory.
    #[arg(short = 's', long, value_name = "DIR")]
    source_dir: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Dumps dependency graph information in JSON.
    DumpDeps {
        /// Package names to start scanning the dependency graph.
        packages: Vec<String>,
    },
    /// Dumps information of packages.
    DumpPackage {
        /// Package names.
        packages: Vec<String>,
    },
    /// Generates a Bazel repository containing overlays and packages.
    GenerateRepo {
        /// Output directory path.
        #[arg(short = 'o', long, value_name = "PATH")]
        output_dir: PathBuf,

        /// Enables verbose output.
        #[arg(short = 'v', long)]
        verbose: bool,
    },
    /// Generates a digest of the repository that can be used to indicate if
    /// any of the overlays, ebuilds, eclasses, etc have changed.
    DigestRepo {
        /// Directory used to store a (file_name, mtime) => digest cache.
        #[command(flatten)]
        args: crate::digest_repo::Args,
    },
}

fn default_source_dir() -> Result<PathBuf> {
    for dir in current_dir()?.ancestors() {
        if dir.join(".repo").exists() {
            return Ok(dir.to_owned());
        }
    }
    bail!(
        "Cannot locate the CrOS source checkout directory from the current directory; \
         consider passing --source-dir option"
    );
}

fn build_override_config_source() -> SimpleConfigSource {
    let source = PathBuf::from("<override>");
    let nodes = vec![
        // HACK: Mask chromeos-base/chromeos-lacros-9999 as it's not functional.
        // TODO: Fix the ebuild and remove this hack.
        ConfigNode {
            source: source.clone(),
            value: ConfigNodeValue::PackageMasks(vec![PackageMaskUpdate {
                kind: PackageMaskKind::Mask,
                atom: "=chromeos-base/chromeos-lacros-9999".parse().unwrap(),
            }]),
        },
        // HACK: Provide packages that are not interesting to install.
        // TODO: Remove this hack.
        ConfigNode {
            source,
            value: ConfigNodeValue::ProvidedPackages(vec![
                // This package was used to force rust binary packages to rebuild.
                // We no longer need this workaround with bazel.
                ProvidedPackage {
                    package_name: "virtual/rust-binaries".to_owned(),
                    version: "0".parse().unwrap(),
                },
                // This is really a BDEPEND and there is no need to declare it as a
                // RDEPEND.
                ProvidedPackage {
                    package_name: "virtual/rust".to_owned(),
                    version: "0".parse().unwrap(),
                },
            ]),
        },
    ];
    SimpleConfigSource::new(nodes)
}

fn setup_tools() -> Result<TempDir> {
    let current_exec = std::env::current_exe()?;

    let tools_dir = tempfile::tempdir()?;

    fs::symlink(current_exec, tools_dir.path().join("ver_test"))?;

    Ok(tools_dir)
}

/// Generates the portage configuration for the board
fn generate_board_config(
    board_root: &Path,
    board: &str,
    repos: &RepositorySet,
    toolchains: &ToolchainConfig,
    translator: &PathTranslator,
) -> Result<()> {
    let files = vec![
        FileOps::symlink (
            "/etc/make.conf",
            "/mnt/host/source/src/third_party/chromiumos-overlay/chromeos/config/make.conf.generic-target",
        ),
        FileOps::symlink (
            "/etc/make.conf.user",
            "/etc/make.conf.user",
        ),
        FileOps::symlink(
            "/etc/portage/make.profile",
            // TODO: Remove hard coded base profile
            translator.to_inner(repos.primary().base_dir())?.join("profiles/base"),
        ),
        // TODO(b/266979761): Remove the need for this list
        FileOps::plainfile("/etc/portage/profile/package.provided", r#"
sys-devel/gcc-10.2.0-r28
sys-libs/glibc-2.33-r17
dev-lang/go-1.18-r2
"#),
    ];
    execute_file_ops(&files, board_root)?;

    let board_etc = board_root.join("etc");

    generate_make_conf_for_board(board, repos, toolchains, translator, &board_etc)?;

    Ok(())
}

/// Generates the portage config for the host SDK
///
/// Instead of depending on an extracted SDK tarball, we hard code the config
/// here. The host config is relatively simple, so it shouldn't be changing
/// that often.
fn generate_host_config(root: &Path) -> Result<()> {
    let ops = vec![
        // Host specific files
        FileOps::symlink(
            "/etc/make.conf",
            "/mnt/host/source/src/third_party/chromiumos-overlay/chromeos/config/make.conf.amd64-host",
        ),
        FileOps::plainfile(
            "/etc/make.conf.board_setup",
            r#"
# Created by cros_sysroot_utils from --board=amd64-host.
ARCH="amd64"
BOARD_OVERLAY="/mnt/host/source/src/overlays/overlay-amd64-host"
BOARD_USE="amd64-host"
CHOST="x86_64-pc-linux-gnu"
# TODO(b/266973461): Remove hard coded -j
MAKEOPTS="-j32"
PORTDIR_OVERLAY="/mnt/host/source/src/overlays/overlay-amd64-host"
"#,
        ),
        FileOps::plainfile("/etc/make.conf.host_setup", ""),
        FileOps::plainfile("/etc/make.conf.user", ""),
        FileOps::symlink(
            "/etc/portage/make.profile",
            "/mnt/host/source/src/overlays/overlay-amd64-host/profiles/base",
        ),
    ];

    execute_file_ops(&ops, root)
}

pub fn alchemist_main(args: Args) -> Result<()> {
    let source_dir = match args.source_dir {
        Some(s) => PathBuf::from(s),
        None => default_source_dir()?,
    };
    let src_dir = source_dir.join("src");

    // Commands that don't need the chroot
    match args.command {
        Commands::DigestRepo { args: local_args } => {
            return digest_repo_main(&args.board, &source_dir, local_args);
        }
        _ => {
            // Handle the rest below
        }
    }

    let translator = enter_fake_chroot(&source_dir, &|new_root_dir, translator| {
        generate_host_config(new_root_dir)?;

        // We throw away the repos and toolchain after we generate the files so we can
        // create new instances that have the "internal" paths instead.
        // TODO: Re-evaluate if this is really necessary.
        let lookup = RepositoryLookup::new(
            &source_dir,
            vec!["src/private-overlays", "src/overlays", "src/third_party"],
        )?;

        let repos = lookup.create_repository_set(&args.board)?;

        let toolchains = load_toolchains(&repos)?;

        let board_root = new_root_dir.join("build").join(&args.board);
        generate_board_config(&board_root, &args.board, &repos, &toolchains, translator)?;
        Ok(())
    })?;

    let root_dir = PathBuf::from("/build").join(&args.board);

    // Load repositories.
    let repos = RepositorySet::load(&root_dir)?;

    // Load configurations.
    let profile = Profile::load_default(&root_dir, &repos)?;
    let site_settings = SiteSettings::load(&root_dir)?;
    let override_source = build_override_config_source();
    let config = ConfigBundle::from_sources(vec![
        // The order matters.
        Box::new(profile) as Box<dyn ConfigSource>,
        Box::new(site_settings) as Box<dyn ConfigSource>,
        Box::new(override_source) as Box<dyn ConfigSource>,
    ]);

    // Set up the package loader
    let tools_dir = setup_tools()?;

    // TODO: Avoid cloning ConfigBundle.
    let loader = CachedPackageLoader::new(PackageLoader::new(
        repos.clone(),
        config.clone(),
        tools_dir.path(),
    ));

    let resolver = PackageResolver::new(
        &repos,
        &config,
        &loader,
        alchemist::ebuild::Stability::Unstable,
    );

    match args.command {
        Commands::DumpDeps { packages } => {
            let starts = packages
                .iter()
                .map(|raw| raw.parse::<PackageAtomDependency>())
                .collect::<Result<Vec<_>>>()?;
            dump_deps_main(&resolver, starts, &src_dir)?;
        }
        Commands::DumpPackage { packages } => {
            let atoms = packages
                .iter()
                .map(|raw| raw.parse::<PackageAtomDependency>())
                .collect::<Result<Vec<_>>>()?;
            dump_package_main(&resolver, atoms)?;
        }
        Commands::GenerateRepo {
            output_dir,
            verbose,
        } => {
            let toolchains = load_toolchains(&repos)?;

            generate_repo_main(
                &args.board,
                &repos,
                &loader,
                &resolver,
                &translator,
                &toolchains,
                &src_dir,
                &output_dir,
                verbose,
            )?;
        }
        Commands::DigestRepo { args: _ } => {
            panic!("BUG: Should be handled above");
        }
    }

    Ok(())
}
