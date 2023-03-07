// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    sync::Arc,
};

use alchemist::{analyze::source::PackageLocalSource, ebuild::PackageDetails};
use anyhow::Result;
use itertools::Itertools;
use lazy_static::lazy_static;
use rayon::prelude::*;
use serde::Serialize;
use tera::Tera;

use crate::generate_repo::common::{DistFileEntry, Package, AUTOGENERATE_NOTICE, CHROOT_SRC_DIR};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template(
            "package.BUILD.bazel",
            include_str!("templates/package.BUILD.bazel"),
        )
        .unwrap();
        tera
    };
}

// Packages that are used to bootstrap the board's SDK
static PRIMORDIAL_PACKAGES: &[&str] = &[
    "sys-kernel/linux-headers",
    "sys-libs/gcc-libs",
    "sys-libs/libcxx",
    "sys-libs/llvm-libunwind",
];

#[derive(Serialize)]
pub struct EBuildEntry {
    ebuild_name: String,
    version: String,
    sources: Vec<String>,
    git_trees: Vec<String>,
    dists: Vec<DistFileEntry>,
    build_deps: Vec<String>,
    runtime_deps: Vec<String>,
    install_set: Vec<String>,
    sdk: String,
    binary_package_src: Option<String>,
}

impl EBuildEntry {
    pub fn try_new(package: &Package) -> Result<Self> {
        let ebuild_name = package
            .details
            .ebuild_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let version = package.details.version.to_string();

        let sources = package
            .sources
            .local_sources
            .iter()
            .map(|source| match source {
                PackageLocalSource::Src(src) => {
                    format!("//internal/sources/{}:__tarballs__", src.to_string_lossy())
                }
                PackageLocalSource::Chrome(version) => format!("@chrome-{version}//:src"),
                PackageLocalSource::Chromite => "@chromite//:src".to_string(),
            })
            .collect();

        let git_trees = package
            .sources
            .repo_sources
            .iter()
            .map(|source| format!("@{}//:src", source.name.to_owned()))
            .collect();

        let dists = package
            .sources
            .dist_sources
            .iter()
            .map(DistFileEntry::try_new)
            .collect::<Result<_>>()?;

        let format_dependencies = |deps: &[Arc<PackageDetails>]| -> Result<Vec<String>> {
            let targets = deps
                .iter()
                .map(|details| {
                    let rel_path = details
                        .ebuild_path
                        .strip_prefix(CHROOT_SRC_DIR)?
                        .parent()
                        .unwrap();
                    Ok(format!(
                        "//internal/packages/{}:{}",
                        rel_path.to_string_lossy(),
                        details.version
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(targets.into_iter().sorted().dedup().collect())
        };

        let build_deps = format_dependencies(&package.dependencies.build_deps)?;
        let runtime_deps = format_dependencies(&package.dependencies.runtime_deps)?;

        let install_set = format_dependencies(&package.install_set)?;

        let sdk = if PRIMORDIAL_PACKAGES
            .iter()
            .any(|v| v == &package.details.package_name)
        {
            "//internal/sdk:base"
        } else {
            "//internal/sdk"
        }
        .to_owned();

        // HACK: Some packages don't build yet. To unblock the prototype effort
        // we just use prebuilt binaries for them.
        let binary_package_src = match package.details.package_name.as_str() {
            // Uses sudo and qemu (b/262458823).
            "chromeos-base/chromeos-fonts" => {
                Some("@arm64_generic_chromeos_fonts_0_0_1_r52//file".to_owned())
            }
            _ => None,
        };

        Ok(Self {
            ebuild_name,
            version,
            sources,
            git_trees,
            dists,
            build_deps,
            runtime_deps,
            install_set,
            sdk,
            binary_package_src,
        })
    }
}

#[derive(Serialize)]
struct BuildTemplateContext {
    ebuilds: Vec<EBuildEntry>,
}

struct PackagesInDir<'a> {
    packages: Vec<&'a Package>,
    original_dir: PathBuf,
}

fn generate_package_build_file(packages_in_dir: &PackagesInDir, out: &Path) -> Result<()> {
    let context = BuildTemplateContext {
        ebuilds: packages_in_dir
            .packages
            .iter()
            .map(|package| EBuildEntry::try_new(package))
            .collect::<Result<_>>()?,
    };

    let mut file = File::create(out)?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "package.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;
    Ok(())
}

fn generate_package(packages_in_dir: &PackagesInDir, output_dir: &Path) -> Result<()> {
    create_dir_all(output_dir)?;

    // Create `*.ebuild` symlinks.
    for package in packages_in_dir.packages.iter() {
        let details = &package.details;
        let file_name = details
            .ebuild_path
            .file_name()
            .expect("ebuild must have a file name");
        symlink(
            packages_in_dir.original_dir.join(&file_name),
            output_dir.join(&file_name),
        )?;
    }

    // Create a `files` symlink if necessary.
    let input_files_dir = packages_in_dir.original_dir.join("files");
    if input_files_dir.try_exists()? {
        let output_files_dir = output_dir.join("files");
        symlink(input_files_dir, output_files_dir)?;
    }

    generate_package_build_file(packages_in_dir, &output_dir.join("BUILD.bazel"))?;

    Ok(())
}

fn join_by_package_dir<'p>(
    all_packages: &'p [Package],
    src_dir: &Path,
) -> HashMap<PathBuf, PackagesInDir<'p>> {
    let mut packages_by_dir = HashMap::<PathBuf, PackagesInDir>::new();

    for package in all_packages.iter() {
        let relative_package_dir = match package.details.ebuild_path.strip_prefix(CHROOT_SRC_DIR) {
            Ok(relative_ebuild_path) => relative_ebuild_path.parent().unwrap().to_owned(),
            Err(_) => continue,
        };
        packages_by_dir
            .entry(relative_package_dir)
            .or_insert_with_key(|relative_package_dir| PackagesInDir {
                packages: Vec::new(),
                original_dir: src_dir.join(relative_package_dir),
            })
            .packages
            .push(package);
    }

    packages_by_dir
}

pub fn generate_internal_packages(
    src_dir: &Path,
    all_packages: &[Package],
    output_dir: &Path,
) -> Result<()> {
    let output_packages_dir = output_dir.join("internal/packages");

    // Generate packages in parallel.
    let packages_by_dir = join_by_package_dir(all_packages, src_dir);
    packages_by_dir
        .into_par_iter()
        .try_for_each(|(relative_package_dir, packages_in_dir)| {
            let output_package_dir = output_packages_dir.join(relative_package_dir);
            generate_package(&packages_in_dir, &output_package_dir)
        })
}
