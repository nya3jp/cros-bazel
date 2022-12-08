// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod dump_deps;
mod dump_package;

use std::path::PathBuf;

use alchemist::{
    dependency::package::PackageAtomDependency, fakechroot::enter_fake_chroot, resolver::Resolver,
};
use anyhow::Result;
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
    #[arg(short = 's', long, value_name = "DIR")]
    source_dir: String,

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

fn main() -> Result<()> {
    let args = Args::parse();

    enter_fake_chroot(&PathBuf::from(args.source_dir))?;

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
