// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use chrono::Datelike;
use clap::Parser;
use cliutil::cli_main;
use extract_package_from_manifest_package::package::{Package, PackageUid};
use extract_package_from_manifest_package::package_set::PackageSet;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs::File, io::Write, path::PathBuf, process::ExitCode};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    #[arg(long, help = "The command to execute to regenerate the manifest")]
    regenerate_command: String,

    #[arg(
        long,
        help = "The path to a json file containing Vec<BinaryPackageInfo>"
    )]
    binary_package_infos: PathBuf,

    #[arg(
        long,
        required = true,
        help = "The path to a .bzl file that we write the manifest to"
    )]
    manifest_out: PathBuf,

    #[arg(
        long,
        help = "Updates the corresponding variable in the manifest file."
    )]
    manifest_variable: String,

    #[arg(
        long,
        help = "Similar to $LD_LIBRARY_PATH, but regexes instead of files."
    )]
    pub ld_library_path_regex: Vec<Regex>,

    #[arg(
        long,
        help = "A regex matching all header file directories we care about."
    )]
    pub header_file_dir_regex: Vec<Regex>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct Manifest {
    root_package: PackageUid,
    packages: Vec<Package>,
    header_file_dirs: BTreeSet<PathBuf>,
    header_file_dir_regexes: Vec<String>,
    ld_library_path: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct BinaryPackageInfo {
    category: String,
    package_name: String,
    version: String,
    slot: String,
    uri: String,
    // Each entry in here is the URI of the dep.
    direct_runtime_deps: Vec<String>,
}

fn do_main() -> Result<()> {
    let args = Cli::parse();

    let binpkgs: Vec<BinaryPackageInfo> =
        serde_json::from_reader(File::open(args.binary_package_infos)?)?;

    let out = fileutil::SafeTempDir::new()?;
    let mut package_set = PackageSet::create(
        &out.path().join("extracted"),
        &binpkgs
            .iter()
            .map(|pkg| PathBuf::from(&pkg.uri))
            .collect::<Vec<_>>(),
    )?;

    let header_file_dirs = package_set.fill_headers(&args.header_file_dir_regex)?;

    let ld_library_path = package_set.generate_ld_library_path(&args.ld_library_path_regex)?;
    package_set.fill_shared_libraries(&ld_library_path)?;
    package_set.wrap_elf_files(&ld_library_path)?;

    let mut packages = package_set.into_packages();
    let root_package = packages[0].uid.clone();
    // While the ordering is deterministic without this, it isn't stable.
    // It's generally filled based on depset ordering, which is deterministic but unspecified.
    // Let's suppose that they chose preorder traversal, and I have a dependency graph
    // a -> b -> c -> d, then add a dependency from a to d.
    // This will result in the manifest file changing from a, b, c, d to a, b, d, c.
    // However, the manifest file shouldn't actually need to change here.
    packages.sort();

    let manifest = Manifest {
        root_package,
        packages,
        header_file_dirs,
        header_file_dir_regexes: args
            .header_file_dir_regex
            .iter()
            .map(|r| r.as_str().to_string())
            .collect(),
        ld_library_path,
    };

    let mut f = std::fs::File::create(&args.manifest_out).with_context(|| {
        format!(
            "Error while trying to open {:?} for writing",
            &args.manifest_out
        )
    })?;
    let year = chrono::Utc::now().date_naive().year();
    f.write_fmt(format_args!(
        "# Copyright {year} The ChromiumOS Authors\n\
        # Use of this source code is governed by a BSD-style license that can be\n\
        # found in the LICENSE file.\n\
        \n\
        # AUTO GENERATED DO NOT EDIT!\n\
        # Regenerate this file using the following command:\n\
        # {}\n\
        # However, you should never need to run this unless\n\
        # bazel explicitly tells you to.\n\
        \n\
        # These three lines ensures that the following json is valid skylark.\n\
        false = False\n\
        true = True\n\
        null = None\n\
        \n\
        {} = ",
        &args.regenerate_command, &args.manifest_variable,
    ))?;
    // Because we're writing to a bzl file instead of a json file, if we don't use an indent of 4,
    // then when we try and submit it, it complains that you need to run cros format on the file.
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut f, formatter);
    // JSON and python dicts are slightly different, so we need to be careful.
    // For example, Option::None serializes to 'null', but we want 'None'. To solve this, we use
    // #[serde(rename = "args", skip_serializing_if = "Option::is_none")]
    manifest.serialize(&mut ser).unwrap();
    f.write(b"\n")?;
    Ok(())
}

fn main() -> ExitCode {
    cli_main(do_main, Default::default())
}
