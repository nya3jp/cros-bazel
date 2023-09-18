// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use chrono::Datelike;
use clap::Parser;
use cliutil::cli_main;
use extract_package_from_manifest_package::{
    filters::filter_header_files,
    package::{Package, PackageCommonArgs, PackageUid},
};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    ffi::OsStr,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    #[arg(long, help = "The command to execute to regenerate the manifest")]
    regenerate_command: String,

    #[arg(long, required=true, num_args=1.., help="The binary packages to unpack")]
    binpkg: Vec<PathBuf>,

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

    #[command(flatten)]
    common: PackageCommonArgs,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct Manifest {
    root_package: PackageUid,
    packages: Vec<Package>,
    header_file_dirs: BTreeSet<PathBuf>,
}

/// Validates that:
/// * No two packages have the same identifier.
/// * No two packages contain the same file
/// * No two packages contain the same shared library (only considering filenames).
///   For example, if one contains /foo/a.so and the other contains /bar/a.so, and /foo and /bar
///   are both shared library directories, then it would be considered an error.
fn validate_packages(packages: &[Package]) -> Result<()> {
    let mut unique_packages: BTreeSet<&PackageUid> = BTreeSet::new();
    for pkg in packages {
        if !unique_packages.insert(&pkg.uid) {
            bail!("Found multiple tbz2 files for package {:?}", pkg.uid)
        }
    }

    let mut file_owners: HashMap<&Path, &PackageUid> = HashMap::new();
    let mut shared_libraries: HashMap<&OsStr, (&Path, &PackageUid)> = HashMap::new();
    for pkg in packages {
        for path in pkg.tarball_content.all_files() {
            if let Some(old_owner) = file_owners.insert(&path, &pkg.uid) {
                bail!(
                    "Conflict: Packages {:?} and {:?} both create file {path:?}",
                    old_owner,
                    pkg.uid
                );
            }
        }

        for lib in &pkg.shared_libraries {
            let name = lib.file_name().expect("File must have a name");
            if let Some((old_path, old_pkg)) = shared_libraries.insert(name, (lib, &pkg.uid)) {
                // If they're the same package, allow masking to occur based on the ordering of
                // shared library regexes.
                // If they're different packages, masking doesn't work well with filter_packages.
                if &pkg.uid != old_pkg {
                    bail!(
                        "Two packages define the shared library {name:?}.
                        {old_pkg:?} generates {old_path:?} and {:?} generates {lib:?}",
                        pkg.uid
                    );
                }
            }
        }
    }
    Ok(())
}

fn do_main() -> Result<()> {
    let args = Cli::parse();

    let out = fileutil::SafeTempDir::new()?;
    let mut packages = args
        .binpkg
        .into_par_iter()
        .map(|path| Package::create(&path, out.path(), &args.common))
        .collect::<Result<Vec<Package>>>()?;

    let root_package = packages[0].uid.clone();
    // While the ordering is deterministic without this, it isn't stable.
    // It's generally filled based on depset ordering, which is deterministic but unspecified.
    // Let's suppose that they chose preorder traversal, and I have a dependency graph
    // a -> b -> c -> d, then add a dependency from a to d.
    // This will result in the manifest file changing from a, b, c, d to a, b, d, c.
    // However, the manifest file shouldn't actually need to change here.
    packages.sort();

    validate_packages(&packages)?;

    let all_files: Vec<&Path> = packages
        .iter()
        .map(|p| p.tarball_content.all_files())
        .flatten()
        .collect();

    let header_file_dirs =
        filter_header_files(&all_files, &args.common.header_file_dir_regex)?.header_file_dirs;

    let manifest = Manifest {
        root_package,
        packages,
        header_file_dirs,
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

#[cfg(test)]
mod tests {
    use super::*;

    use extract_package_from_manifest_package::package::TarballContent;

    #[test]
    fn duplicate_packages() -> Result<()> {
        let empty_package = |name: &str| Package {
            uid: PackageUid {
                name: name.to_string(),
                slot: "0/0".to_string(),
            },
            tarball_content: Default::default(),
            header_files: vec![],
            shared_libraries: vec![],
        };
        validate_packages(&[empty_package("a"), empty_package("b")])?;
        assert!(validate_packages(&[empty_package("a"), empty_package("a"),]).is_err());
        Ok(())
    }

    #[test]
    fn duplicate_files() -> Result<()> {
        let gen_package = |name: &str, content: TarballContent| Package {
            uid: PackageUid {
                name: name.to_string(),
                slot: "0/0".to_string(),
            },
            tarball_content: content,
            header_files: vec![],
            shared_libraries: vec![],
        };
        validate_packages(&[
            gen_package(
                "a",
                TarballContent {
                    files: vec![PathBuf::from("a/file")],
                    symlinks: vec![PathBuf::from("a/symlink")],
                },
            ),
            gen_package(
                "b",
                TarballContent {
                    files: vec![PathBuf::from("b/file")],
                    symlinks: vec![PathBuf::from("b/symlink")],
                },
            ),
        ])?;
        assert!(validate_packages(&[
            gen_package(
                "a",
                TarballContent {
                    files: vec![PathBuf::from("overlap")],
                    symlinks: vec![],
                }
            ),
            gen_package(
                "b",
                TarballContent {
                    files: vec![],
                    symlinks: vec![PathBuf::from("overlap")],
                }
            ),
        ])
        .is_err());
        Ok(())
    }

    #[test]
    fn duplicate_shared_libraries() -> Result<()> {
        let gen_package = |name: &str, shared_libraries| Package {
            uid: PackageUid {
                name: name.to_string(),
                slot: "0/0".to_string(),
            },
            tarball_content: Default::default(),
            header_files: vec![],
            shared_libraries,
        };
        validate_packages(&[
            gen_package("a", vec![PathBuf::from("a/foo.so")]),
            gen_package("b", vec![PathBuf::from("b/bar.so")]),
        ])?;
        assert!(validate_packages(&[
            gen_package("a", vec![PathBuf::from("a/foo.so")]),
            gen_package("b", vec![PathBuf::from("b/foo.so")]),
        ])
        .is_err());
        Ok(())
    }
}
