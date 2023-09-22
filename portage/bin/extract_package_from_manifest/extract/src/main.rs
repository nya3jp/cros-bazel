// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use clap::Parser;
use cliutil::cli_main;
use extract_package_from_manifest_package::package::Package;
use extract_package_from_manifest_package::package_set::PackageSet;
use regex::Regex;
use std::{fs::File, io::BufReader, path::PathBuf, process::ExitCode};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    #[arg(
        long,
        help = "The command to execute to fix an incorrect set of files in \
        the interface"
    )]
    regenerate_command: String,

    #[arg(long, required = true, help = "The binary package to unpack")]
    binpkg: PathBuf,

    #[arg(
        long,
        required = true,
        help = "The path to a json file containing the expected package contents"
    )]
    manifest: PathBuf,

    #[arg(long, help = "The directory to output to")]
    out_dir: PathBuf,

    #[arg(long, help = "Equivalent to the bash variable $LD_LIBRARY_PATH.")]
    ld_library_path: Vec<PathBuf>,

    #[arg(
        long,
        help = "A regex matching all header file directories we care about."
    )]
    pub header_file_dir_regex: Vec<Regex>,
}

fn do_main() -> Result<()> {
    let args = Cli::parse();

    let mut package_set = PackageSet::create(&args.out_dir, &[args.binpkg.as_path()])?;
    package_set.fill_headers(&args.header_file_dir_regex)?;
    package_set.fill_shared_libraries(&args.ld_library_path)?;
    package_set.wrap_elf_files(&args.ld_library_path)?;

    let got_pkg = &package_set.into_packages()[0];

    let want_pkg: Package = serde_json::from_reader(BufReader::new(
        File::open(&args.manifest)
            .with_context(|| format!("Failed to open {:?}", args.manifest))?,
    ))?;

    if *got_pkg != want_pkg {
        eprintln!();
        // A change here requires a change to
        // extract_package_from_manifest/extract.bzl and .bazel_fix_commands.json.
        bail!(
            "\n\
            Interface for binary has changed. Please run '{}'\n\
            Consider using bazel-watcher (ibazel) to automatically \
            apply the fix and rerun \
            (https://github.com/bazelbuild/bazel-watcher).\n",
            &args.regenerate_command
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    cli_main(do_main, Default::default())
}
