// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod dump_deps;
mod dump_package;

use std::{env::current_dir, path::PathBuf};

use alchemist::{
    dependency::package::PackageAtomDependency, fakechroot::enter_fake_chroot, resolver::Resolver,
};
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use dump_deps::dump_deps_main;
use dump_package::dump_package_main;

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

fn main() -> Result<()> {
    let args = Args::parse();

    let source_dir = match args.source_dir {
        Some(s) => PathBuf::from(s),
        None => default_source_dir()?,
    };
    enter_fake_chroot(&source_dir)?;

    let root_dir = PathBuf::from("/build").join(&args.board);
    let tools_dir = std::env::current_exe()?.parent().unwrap().to_owned();
    let resolver = Resolver::load(
        &root_dir,
        &tools_dir,
        alchemist::ebuild::Stability::Unstable,
    )?;

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
    }

    Ok(())
}
