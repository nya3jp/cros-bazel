// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod dump_deps;
mod dump_package;
mod generate_repo;

use std::{env::current_dir, path::PathBuf};

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
};
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use dump_deps::dump_deps_main;
use dump_package::dump_package_main;
use generate_repo::generate_repo_main;

#[derive(Parser)]
#[command(name = "alchemist")]
#[command(author = "ChromiumOS Authors")]
#[command(about = "Analyzes Portage trees", long_about = None)]
struct Args {
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

#[derive(Subcommand)]
enum Commands {
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
            source: source.clone(),
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

fn main() -> Result<()> {
    let args = Args::parse();

    let source_dir = match args.source_dir {
        Some(s) => PathBuf::from(s),
        None => default_source_dir()?,
    };
    let translator = enter_fake_chroot(&source_dir)?;

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

    // Set up the package loader.
    let tools_dir = std::env::current_exe()?.parent().unwrap().to_owned();
    // TODO: Avoid cloning ConfigBundle.
    let loader = CachedPackageLoader::new(PackageLoader::new(
        repos.clone(),
        config.clone(),
        &tools_dir,
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
            dump_deps_main(&resolver, starts)?;
        }
        Commands::DumpPackage { packages } => {
            let atoms = packages
                .iter()
                .map(|raw| raw.parse::<PackageAtomDependency>())
                .collect::<Result<Vec<_>>>()?;
            dump_package_main(&resolver, atoms)?;
        }
        Commands::GenerateRepo { output_dir } => {
            generate_repo_main(
                &args.board,
                &repos,
                &loader,
                &resolver,
                &translator,
                &output_dir,
            )?;
        }
    }

    Ok(())
}
