// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod compare_packages;
#[cfg(test)]
mod testdata;
mod validate_package;

use anyhow::Result;
use binarypackage::BinaryPackage;
use clap::{Parser, Subcommand};
use cliutil::{cli_main, ConfigBuilder};
use itertools::Itertools;

use crate::compare_packages::{do_compare_packages, ComparePackagesArgs};
use crate::validate_package::{do_validate_package, ValidatePackageArgs};
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser, Debug)]
#[command()]
struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ExtractXpak(ExtractXpakArgs),
    ComparePackages(ComparePackagesArgs),
    ValidatePackage(ValidatePackageArgs),
}

/// Shows XPAK entries in a Portage binary package file.
#[derive(Parser, Debug)]
struct ExtractXpakArgs {
    /// Save raw XPAK entries to the specified directory instead of
    /// showing them in the console.
    #[arg(long)]
    dump: Option<PathBuf>,

    /// Portage binary package file.
    #[arg()]
    binary_package: PathBuf,
}

fn do_main() -> Result<()> {
    let cli = Cli::try_parse()?;
    match cli.commands {
        Commands::ExtractXpak(args) => do_extract_xpak(args),
        Commands::ComparePackages(args) => do_compare_packages(args),
        Commands::ValidatePackage(args) => do_validate_package(args),
    }
}

fn do_extract_xpak(args: ExtractXpakArgs) -> Result<()> {
    let pkg = BinaryPackage::open(&args.binary_package)?;

    if let Some(dump_dir) = args.dump {
        // Dump to the specified directory.
        std::fs::create_dir_all(&dump_dir)?;
        for (key, value) in pkg.xpak() {
            std::fs::write(dump_dir.join(key), value)?;
        }
    } else {
        // Show in the console.
        let entries = pkg.xpak().iter().sorted_by(|a, b| a.0.cmp(b.0));
        for (key, value) in entries {
            println!("{key}:");
            if let Ok(utf8_value) = String::from_utf8(value.clone()) {
                let utf8_value = utf8_value.strip_suffix('\n').unwrap_or(&utf8_value);
                for line in utf8_value.split('\n') {
                    println!("\t{line}");
                }
            } else {
                println!("\t<binary>");
            }
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    cli_main(
        do_main,
        ConfigBuilder::new()
            .log_command_line(false)
            .build()
            .expect("valid config"),
    )
}
