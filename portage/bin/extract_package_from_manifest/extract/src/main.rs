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
    /// The command to execute to fix an incorrect set of files in
    /// the interface
    #[arg(long)]
    regenerate_command: String,

    /// The binary package to unpack
    #[arg(long, required = true)]
    binpkg: PathBuf,

    /// The path to a json file containing the expected package contents
    #[arg(long, required = true)]
    manifest: PathBuf,

    /// The directory to output to
    #[arg(long)]
    out_dir: PathBuf,

    /// Equivalent to the bash variable $LD_LIBRARY_PATH.
    #[arg(long)]
    ld_library_path: Vec<PathBuf>,

    /// A regex matching all header file directories we care about.
    #[arg(long)]
    pub header_file_dir_regex: Vec<Regex>,
}

fn do_main() -> Result<()> {
    let args = Cli::try_parse()?;

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
