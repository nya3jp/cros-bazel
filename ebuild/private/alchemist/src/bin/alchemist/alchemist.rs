// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::os::unix::fs;
use std::path::Path;
use std::sync::Arc;

use std::{env::current_dir, path::PathBuf};

use crate::digest_repo::digest_repo_main;
use crate::dump_package::dump_package_main;
use crate::generate_repo::generate_repo_main;

use alchemist::common::is_inside_chroot;
use alchemist::toolchain::ToolchainConfig;
use alchemist::{
    config::{
        bundle::ConfigBundle, profile::Profile, site::SiteSettings, ConfigNode, ConfigNodeValue,
        ConfigSource, PackageMaskKind, PackageMaskUpdate, ProvidedPackage, SimpleConfigSource,
    },
    dependency::package::PackageAtom,
    ebuild::{CachedPackageLoader, PackageLoader},
    fakechroot::{enter_fake_chroot, PathTranslator},
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

    /// Profile of the board.
    #[arg(short = 'p', long, value_name = "PROFILE", default_value = "base")]
    profile: String,

    /// Path to the ChromiumOS source directory root.
    /// If unset, it is inferred from the current directory.
    #[arg(short = 's', long, value_name = "DIR")]
    source_dir: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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

    fs::symlink(&current_exec, tools_dir.path().join("ver_test"))?;
    fs::symlink(&current_exec, tools_dir.path().join("ver_rs"))?;

    Ok(tools_dir)
}

/// Container that contains all the data structures for a specific board.
pub struct TargetData {
    pub board: String,
    pub profile: String,
    pub repos: Arc<RepositorySet>,
    pub config: Arc<ConfigBundle>,
    pub loader: Arc<CachedPackageLoader>,
    pub resolver: PackageResolver,
    pub toolchains: ToolchainConfig,
}

fn load_board(
    board: &str,
    profile_name: &str,
    root_dir: &Path,
    tools_dir: &Path,
) -> Result<TargetData> {
    // Load repositories.
    let repos = Arc::new(RepositorySet::load(root_dir)?);

    // Load configurations.
    let config = Arc::new({
        let profile = Profile::load_default(root_dir, &repos)?;
        let site_settings = SiteSettings::load(root_dir)?;
        let override_source = build_override_config_source();

        ConfigBundle::from_sources(vec![
            // The order matters.
            Box::new(profile) as Box<dyn ConfigSource>,
            Box::new(site_settings) as Box<dyn ConfigSource>,
            Box::new(override_source) as Box<dyn ConfigSource>,
        ])
    });

    // Force accept 9999 ebuilds when running outside a cros chroot.
    let force_accept_9999_ebuilds = !is_inside_chroot()?;

    let loader = Arc::new(CachedPackageLoader::new(PackageLoader::new(
        Arc::clone(&repos),
        Arc::clone(&config),
        tools_dir,
        force_accept_9999_ebuilds,
    )));

    let resolver =
        PackageResolver::new(Arc::clone(&repos), Arc::clone(&config), Arc::clone(&loader));

    let toolchains = load_toolchains(&repos)?;

    Ok(TargetData {
        board: board.to_string(),
        profile: profile_name.to_string(),
        repos,
        config,
        loader,
        resolver,
        toolchains,
    })
}

pub fn alchemist_main(args: Args) -> Result<()> {
    let source_dir = match args.source_dir {
        Some(s) => PathBuf::from(s),
        None => default_source_dir()?,
    };
    let src_dir = source_dir.join("src");

    // Enter a fake chroot when running outside a cros chroot.
    let translator = if is_inside_chroot()? {
        PathTranslator::noop()
    } else {
        enter_fake_chroot(&args.board, &args.profile, &source_dir)?
    };

    let tools_dir = setup_tools()?;

    let target = load_board(
        &args.board,
        &args.profile,
        &Path::new("/build").join(&args.board),
        tools_dir.path(),
    )?;

    match args.command {
        Commands::DumpPackage { packages } => {
            let atoms = packages
                .iter()
                .map(|raw| raw.parse::<PackageAtom>())
                .collect::<Result<Vec<_>>>()?;
            dump_package_main(&target.resolver, atoms)?;
        }
        Commands::GenerateRepo { output_dir } => {
            generate_repo_main(target, &translator, &src_dir, &output_dir)?;
        }
        Commands::DigestRepo { args: local_args } => {
            digest_repo_main(&args.board, &source_dir, local_args)?;
        }
    }

    Ok(())
}
