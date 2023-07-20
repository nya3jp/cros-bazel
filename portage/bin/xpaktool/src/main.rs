// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use binarypackage::BinaryPackage;
use clap::Parser;
use cliutil::cli_main;
use itertools::Itertools;
use std::{path::PathBuf, process::ExitCode};

/// Shows XPAK entries in a Portage binary package file.
#[derive(Parser, Debug)]
struct Args {
    #[arg(
        long,
        help = "Save raw XPAK entries to the specified directory instead of \
                showing them in the console."
    )]
    dump: Option<PathBuf>,

    #[arg(help = "Portage binary package file.")]
    binary_package: PathBuf,
}

fn do_main() -> Result<()> {
    let args = Args::parse();

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
    cli_main(do_main)
}
