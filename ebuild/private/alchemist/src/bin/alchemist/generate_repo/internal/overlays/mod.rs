// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use alchemist::{
    analyze::source::PackageLocalSourceOrigin, dependency::package::PackageAtomDependency,
    fakechroot::PathTranslator, resolver::PackageResolver,
};
use anyhow::Result;
use itertools::Itertools;
use rayon::prelude::*;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::generate_repo::common::{DistFileEntry, Package, AUTOGENERATE_NOTICE, CHROOT_SRC_DIR};

static PACKAGE_BUILD_TEMPLATE: &str = include_str!("package-template.BUILD.bazel");

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
    dists: Vec<DistFileEntry>,
    build_deps: Vec<String>,
    runtime_deps: Vec<String>,
    post_deps: Vec<String>,
    sdk: String,
}

impl EBuildEntry {
    pub fn try_new(package: &Package, resolver: &PackageResolver) -> Result<Self> {
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
            .map(|source| {
                let repo_name = match source.origin {
                    PackageLocalSourceOrigin::Src => "@",
                    PackageLocalSourceOrigin::Chrome => "@chrome",
                    PackageLocalSourceOrigin::Chromite => "@chromite",
                };
                format!("{}//{}:src", repo_name, source.path)
            })
            .collect();

        let dists = package
            .sources
            .remote_sources
            .iter()
            .map(DistFileEntry::try_new)
            .collect::<Result<_>>()?;

        let resolve_dependencies = |deps: &Vec<PackageAtomDependency>| -> Result<Vec<String>> {
            let targets = deps
                .iter()
                .map(|atom| {
                    let package = resolver.find_best_package(atom)?;
                    let rel_path = package
                        .ebuild_path
                        .strip_prefix(CHROOT_SRC_DIR)?
                        .parent()
                        .unwrap();
                    Ok(format!(
                        "//internal/overlays/{}:{}",
                        rel_path.to_string_lossy(),
                        package.version
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(targets.into_iter().sorted().dedup().collect())
        };

        let build_deps = resolve_dependencies(&package.dependencies.build_deps)?;
        let runtime_deps = resolve_dependencies(&package.dependencies.runtime_deps)?;
        let post_deps = resolve_dependencies(&package.dependencies.post_deps)?;

        let sdk = if PRIMORDIAL_PACKAGES
            .iter()
            .any(|v| v == &package.details.package_name)
        {
            "//internal/sdk:base"
        } else {
            "//internal/sdk"
        }
        .to_owned();

        Ok(Self {
            ebuild_name,
            version,
            sources,
            dists,
            build_deps,
            runtime_deps,
            post_deps,
            sdk,
        })
    }
}

#[derive(Serialize)]
struct BuildTemplateContext {
    ebuilds: Vec<EBuildEntry>,
}

struct PackagesInDir<'a> {
    original_dir: PathBuf,
    packages: Vec<&'a Package>,
}

fn generate_internal_package_build_file(
    packages_in_dir: &PackagesInDir,
    out: &Path,
    resolver: &PackageResolver,
) -> Result<()> {
    let context = BuildTemplateContext {
        ebuilds: packages_in_dir
            .packages
            .iter()
            .map(|package| EBuildEntry::try_new(*package, resolver))
            .collect::<Result<_>>()?,
    };

    let mut templates = TinyTemplate::new();
    templates.add_template("main", PACKAGE_BUILD_TEMPLATE)?;
    let rendered = templates.render("main", &context)?;

    let mut file = File::create(out)?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    file.write_all(rendered.as_bytes())?;
    Ok(())
}

fn generate_internal_package(
    packages_in_dir: PackagesInDir,
    package_output_dir: &Path,
    translator: &PathTranslator,
    resolver: &PackageResolver,
) -> Result<()> {
    create_dir_all(&package_output_dir)?;

    // Create `*.ebuild` symlinks.
    for package in packages_in_dir.packages.iter() {
        let details = &package.details;
        symlink(
            translator.translate(&details.ebuild_path),
            package_output_dir.join(details.ebuild_path.file_name().unwrap()),
        )?;
    }

    // Create a `files` symlink if necessary.
    // TODO: Create symlinks even if there is no ebuild.
    let input_files_dir = packages_in_dir.original_dir.join("files");
    if input_files_dir.try_exists()? {
        let output_files_dir = package_output_dir.join("files");
        symlink(input_files_dir, output_files_dir)?;
    }

    // Create `*.bashrc` symlinks if necessary.
    // TODO: Create symlinks even if there is no ebuild.
    for entry in packages_in_dir.original_dir.read_dir()? {
        let filename = entry?.file_name();
        if filename.to_string_lossy().ends_with(".bashrc") {
            symlink(
                packages_in_dir.original_dir.join(&filename),
                package_output_dir.join(&filename),
            )?;
        }
    }

    // Generate `BUILD.bazel`.
    generate_internal_package_build_file(
        &packages_in_dir,
        &package_output_dir.join("BUILD.bazel"),
        resolver,
    )
}

fn join_by_package_dir<'a>(
    all_packages: &'a Vec<Package>,
    translator: &PathTranslator,
) -> HashMap<PathBuf, PackagesInDir<'a>> {
    let mut packages_by_dir = HashMap::<PathBuf, PackagesInDir>::new();

    for package in all_packages.iter() {
        let original_dir = translator.translate(package.details.ebuild_path.parent().unwrap());
        let relative_package_dir = match package.details.ebuild_path.strip_prefix(CHROOT_SRC_DIR) {
            Ok(relative_ebuild_path) => relative_ebuild_path.parent().unwrap().to_owned(),
            Err(_) => continue,
        };
        packages_by_dir
            .entry(relative_package_dir)
            .or_insert_with(|| PackagesInDir {
                original_dir,
                packages: Vec::new(),
            })
            .packages
            .push(package);
    }

    packages_by_dir
}

pub fn generate_internal_packages(
    all_packages: &Vec<Package>,
    resolver: &PackageResolver,
    translator: &PathTranslator,
    output_dir: &Path,
) -> Result<()> {
    let packages_by_dir = join_by_package_dir(&all_packages, translator);

    // Generate packages in parallel.
    packages_by_dir
        .into_par_iter()
        .try_for_each(|(relative_package_dir, packages_in_dir)| {
            let package_output_dir = output_dir
                .join("internal/overlays")
                .join(relative_package_dir);
            generate_internal_package(packages_in_dir, &package_output_dir, translator, resolver)
        })
}
