// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use chrono::Datelike;
use clap::Parser;
use cliutil::cli_main;
use extract_package_from_manifest_package::package::{Package, PackageUid};
use extract_package_from_manifest_package::package_set::PackageSet;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashMap},
    fs::File,
    io::Write,
    path::Path,
    path::PathBuf,
    process::Command,
    process::ExitCode,
};

// TODO(b/304662445): Update this to one with more secure configuration.
static GS_PREFIX: &str = "gs://chromeos-bazel-prebuilt-test";

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

    #[arg(
        long,
        help = "If provided, instead of updating a local manifest, updates a manifest stored in a \
        GS bucket."
    )]
    pub remote_prebuilt_name: Option<String>,
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

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct RemoteManifest {
    manifest: Manifest,
    providers: Vec<BinaryPackageInfo>,
}

/// Updates the url in MODULE.bazel for the repo `name` to `url`.
/// If no repo in `name` is found, returns an error containing instructions for the developer.
fn update_module(name: &str, url: &str) -> Result<()> {
    let module_file = Path::new(&std::env::var("BUILD_WORKSPACE_DIRECTORY")?)
        .join("MODULE.bazel")
        .canonicalize()?;
    let content = std::fs::read_to_string(&module_file)?;
    let re = regex::Regex::new(&format!(
        r#"(prebuilt_sdk.from_url\(\s*name = "{name}",\s*url = ")[^"]+(",?\s*\))"#
    ))?;

    if !re.is_match(&content) {
        bail!(
            r#"Unable to find repo {name} in MODULE.bazel. \
            Please add the following to your MODULE.bazel:\

prebuilt_sdk.from_url(
    name = "{name}",
    url = "{url}",
)

prebuilt_sdk_tarballs.from_manifests(
    manifests = ["@{name}//file:manifest.json"],
)
"#
        )
    }
    let new_content = re
        .replace(&content, |caps: &Captures| {
            format!("{}{}{}", &caps[1], url, &caps[2])
        })
        .to_string();
    // Avoid modifying the mtime of MODULE.bazel if nothing's changed, since bazel uses that to
    // determine whether we need to re-execute the modules.
    if new_content != content {
        std::fs::write(module_file, new_content)?;
    }
    Ok(())
}

fn checksum(p: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    std::io::copy(&mut std::fs::File::open(&p)?, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}

fn update_remote_manifest(
    manifest: Manifest,
    out: &Path,
    binpkgs: Vec<BinaryPackageInfo>,
    remote_prebuilt_name: &str,
) -> Result<()> {
    let uri_mapping: HashMap<String, String> = binpkgs
        .iter()
        .map(|binpkg| {
            let main_slot = match binpkg.slot.split_once("/") {
                Some((main, _)) => main,
                None => &binpkg.slot,
            };
            let orig = Path::new(&binpkg.uri).canonicalize()?;
            let dst = format!(
                "prebuilts/{}/{}/slot_{}/{}.tbz2",
                binpkg.category,
                binpkg.package_name,
                main_slot.to_string(),
                checksum(&orig)?
            );
            let abs_dst = out.join(&dst);
            std::fs::create_dir_all(abs_dst.parent().unwrap())?;
            std::os::unix::fs::symlink(orig, abs_dst)?;
            Ok((binpkg.uri.to_string(), format!("{GS_PREFIX}/{dst}")))
        })
        .collect::<Result<_>>()?;

    let remote_manifest = RemoteManifest {
        manifest,
        providers: binpkgs
            .into_iter()
            .map(|binpkg| BinaryPackageInfo {
                uri: uri_mapping.get(&binpkg.uri).unwrap().to_string(),
                direct_runtime_deps: binpkg
                    .direct_runtime_deps
                    .iter()
                    .map(|dep| uri_mapping.get(dep).unwrap().to_string())
                    .collect(),
                ..binpkg
            })
            .collect(),
    };

    let manifest_path = out.join("manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string(&remote_manifest)?)?;
    std::fs::create_dir_all(out.join("manifests"))?;
    let dst = format!("manifests/{}.json", checksum(&manifest_path)?);
    std::fs::rename(manifest_path, out.join(&dst))?;

    let mut cmd = Command::new("gsutil");
    cmd.args(["-m", "rsync", "-r"]).arg(out).arg(GS_PREFIX);
    log::info!("Running {cmd:?}");
    processes::run_and_check(&mut cmd)?;
    update_module(&remote_prebuilt_name, &format!("{GS_PREFIX}/{dst}"))?;
    Ok(())
}

fn update_local_manifest(
    manifest: Manifest,
    out: &Path,
    regenerate_command: &str,
    manifest_variable: &str,
) -> Result<()> {
    let mut f = std::fs::File::create(&out)
        .with_context(|| format!("Error while trying to open {out:?} for writing"))?;
    let year = chrono::Utc::now().date_naive().year();
    f.write_fmt(format_args!(
        "# Copyright {year} The ChromiumOS Authors\n\
        # Use of this source code is governed by a BSD-style license that can be\n\
        # found in the LICENSE file.\n\
        \n\
        # AUTO GENERATED DO NOT EDIT!\n\
        # Regenerate this file using the following command:\n\
        # {regenerate_command}\n\
        # However, you should never need to run this unless\n\
        # bazel explicitly tells you to.\n\
        \n\
        # These three lines ensures that the following json is valid skylark.\n\
        false = False\n\
        true = True\n\
        null = None\n\
        \n\
        {manifest_variable} = ",
    ))?;
    // Because we're writing to a bzl file instead of a json file, if we don't use an indent of
    // 4, then when we try and submit it, it complains that you need to run cros format on the
    // file.
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut f, formatter);
    // JSON and python dicts are slightly different, so we need to be careful.
    // For example, Option::None serializes to 'null', but we want 'None'. To solve this, we use
    // #[serde(rename = "args", skip_serializing_if = "Option::is_none")]
    manifest.serialize(&mut ser).unwrap();
    f.write(b"\n")?;
    Ok(())
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

    if let Some(remote_prebuilt_name) = args.remote_prebuilt_name {
        update_remote_manifest(
            manifest,
            &out.path().join("remote"),
            binpkgs,
            &remote_prebuilt_name,
        )?
    } else {
        update_local_manifest(
            manifest,
            &args.manifest_out,
            &args.regenerate_command,
            &args.manifest_variable,
        )?
    }
    Ok(())
}

fn main() -> ExitCode {
    cli_main(do_main, Default::default())
}
