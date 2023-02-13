// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{create_dir, create_dir_all, File},
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use alchemist::{
    analyze::source::PackageLocalSourceOrigin, dependency::package::PackageAtomDependency,
    repository::RepositorySet, resolver::PackageResolver,
};
use anyhow::{Context, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use rayon::prelude::*;
use serde::Serialize;
use tera::Tera;
use walkdir::WalkDir;

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

/// Mirrors files in the original overlay to the output tree with symlinks.
///
/// This function skips creating symlinks for these files:
/// - `**/BUILD.bazel`: They should not exist in overlays and interferes with
///   `BUILD.bazel` files we will generate later.
/// - `metadata/md5-cache`: They're not consumed by alchemist, and we have too
///   many files under the directory.
fn generate_overlay_symlinks(original_dir: &Path, output_dir: &Path) -> Result<()> {
    let walk = WalkDir::new(original_dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|entry| {
            if entry.file_name() == OsStr::new("BUILD.bazel") {
                return false;
            }
            let relative_path = entry.path().strip_prefix(original_dir).unwrap();
            if relative_path == Path::new("metadata/md5-cache") {
                return false;
            }
            true
        });
    for entry in walk {
        let entry = entry?;
        let original_file = entry.path();
        let relative_path = original_file.strip_prefix(original_dir).unwrap();
        let output_file = output_dir.join(relative_path);
        if entry.file_type().is_dir() {
            create_dir(&output_file).with_context(|| format!("mkdir {}", output_file.display()))?;
            continue;
        }
        symlink(&original_file, &output_file).with_context(|| {
            format!(
                "ln -s {} {}",
                original_file.display(),
                output_file.display()
            )
        })?;
    }
    Ok(())
}

fn generate_overlays(
    repos: &RepositorySet,
    src_dir: &Path,
    output_overlays_dir: &Path,
) -> Result<()> {
    repos
        .get_repos()
        .into_iter()
        .try_for_each(|repo| -> Result<()> {
            let relative_dir = repo.base_dir().strip_prefix("/mnt/host/source/src")?;
            let original_dir = src_dir.join(relative_dir);
            let output_dir = output_overlays_dir.join(relative_dir);

            create_dir_all(&output_dir)
                .with_context(|| format!("mkdir -p {}", output_dir.display()))?;

            generate_overlay_symlinks(&original_dir, &output_dir)?;

            Ok(())
        })?;
    Ok(())
}

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
    binary_package_src: Option<String>,
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
            .map(|source| match source.origin {
                PackageLocalSourceOrigin::Src => {
                    format!("//internal/sources/{}:__tarballs__", source.path)
                }
                PackageLocalSourceOrigin::Chrome => format!("@chrome//{}:src", source.path),
                PackageLocalSourceOrigin::Chromite => format!("@chromite//{}:src", source.path),
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
            dists,
            build_deps,
            runtime_deps,
            post_deps,
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
            .map(|package| EBuildEntry::try_new(package, resolver))
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

fn join_by_package_dir<'a>(all_packages: &'a Vec<Package>) -> HashMap<PathBuf, PackagesInDir<'a>> {
    let mut packages_by_dir = HashMap::<PathBuf, PackagesInDir>::new();

    for package in all_packages.iter() {
        let relative_package_dir = match package.details.ebuild_path.strip_prefix(CHROOT_SRC_DIR) {
            Ok(relative_ebuild_path) => relative_ebuild_path.parent().unwrap().to_owned(),
            Err(_) => continue,
        };
        packages_by_dir
            .entry(relative_package_dir)
            .or_insert_with(|| PackagesInDir {
                packages: Vec::new(),
            })
            .packages
            .push(package);
    }

    packages_by_dir
}

pub fn generate_internal_overlays(
    src_dir: &Path,
    repos: &RepositorySet,
    all_packages: &Vec<Package>,
    resolver: &PackageResolver,
    output_dir: &Path,
) -> Result<()> {
    let output_overlays_dir = output_dir.join("internal/overlays");
    generate_overlays(repos, src_dir, &output_overlays_dir)?;

    // Generate packages in parallel.
    let packages_by_dir = join_by_package_dir(all_packages);
    packages_by_dir
        .into_par_iter()
        .try_for_each(|(relative_package_dir, packages_in_dir)| {
            let output_file = output_overlays_dir
                .join(relative_package_dir)
                .join("BUILD.bazel");
            generate_internal_package_build_file(&packages_in_dir, &output_file, resolver)
        })
}
