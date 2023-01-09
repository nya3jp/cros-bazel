// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    fs::{create_dir_all, remove_dir_all, File},
    io::{ErrorKind, Write},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use anyhow::Result;

use alchemist::{
    analyze::{
        dependency::{analyze_dependencies, PackageDependencies},
        source::{analyze_sources, fixup_sources, PackageRemoteSource, PackageSources},
    },
    dependency::package::PackageAtomDependency,
    ebuild::{CachedEBuildEvaluator, PackageDetails},
    fakechroot::PathTranslator,
    repository::RepositorySet,
    resolver::PackageResolver,
};
use itertools::Itertools;
use rayon::prelude::*;
use serde::Serialize;
use tinytemplate::TinyTemplate;

static CHROOT_SRC_DIR: &str = "/mnt/host/source/src";

static AUTOGENERATE_NOTICE: &str = "# AUTO-GENERATED FILE. DO NOT EDIT.\n\n";

static DEFAULT_MIRRORS: &[&str] = &[
    "https://commondatastorage.googleapis.com/chromeos-mirror/gentoo/distfiles/",
    "https://commondatastorage.googleapis.com/chromeos-localmirror/distfiles/",
];

static ALLOWED_MIRRORS: &[&str] = &[
    "https://commondatastorage.googleapis.com/",
    "https://storage.googleapis.com/",
];

// Packages that are used to bootstrap the board's SDK
static PRIMORDIAL_PACKAGES: &[&str] = &[
    "sys-kernel/linux-headers",
    "sys-libs/gcc-libs",
    "sys-libs/libcxx",
    "sys-libs/llvm-libunwind",
];

struct Package {
    details: Arc<PackageDetails>,
    dependencies: PackageDependencies,
    sources: PackageSources,
}

struct PackagesInDir<'a> {
    input_dir: PathBuf,
    output_dir: PathBuf,
    packages: Vec<&'a Package>,
}

#[derive(Serialize)]
struct DistFileEntry {
    repository_name: String,
    filename: String,
    integrity: String,
    urls: Vec<String>,
}

fn file_name_to_repository_name(file_name: &str) -> String {
    let escaped_file_name: String = file_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c.to_string()
            } else {
                format!("_{:x}_", c as u32)
            }
        })
        .join("");
    format!("portage-dist_{}", escaped_file_name)
}

impl DistFileEntry {
    fn try_new(source: &PackageRemoteSource) -> Result<Self> {
        let special_urls = source
            .urls
            .iter()
            .flat_map(|url| {
                let url_string = {
                    let url = {
                        // Fix duplicated slashes in URL paths.
                        let mut url = url.clone();
                        url.set_path(&url.path().replace("//", "/"));
                        url
                    };
                    // Convert gs:// URLs to https:// URLs.
                    if url.scheme() == "gs" {
                        format!(
                            "https://storage.googleapis.com/{}{}",
                            url.host_str().unwrap_or_default(),
                            url.path()
                        )
                    } else {
                        url.to_string()
                    }
                };
                // Keep allow-listed URLs only.
                if ALLOWED_MIRRORS
                    .iter()
                    .all(|prefix| !url_string.starts_with(prefix))
                {
                    return None;
                }
                Some(url_string)
            })
            .collect_vec();

        let urls = if !special_urls.is_empty() {
            special_urls
        } else {
            DEFAULT_MIRRORS
                .iter()
                .map(|prefix| format!("{}{}", prefix, source.filename))
                .collect_vec()
        };

        Ok(Self {
            repository_name: file_name_to_repository_name(&source.filename),
            filename: source.filename.clone(),
            integrity: source.compute_integrity()?,
            urls,
        })
    }
}

#[derive(Serialize)]
struct EBuildEntry {
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
    fn try_new(board: &str, package: &Package, resolver: &PackageResolver) -> Result<Self> {
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
            .map(|label| {
                if label.starts_with("@") {
                    label.clone()
                } else {
                    format!("@{}", label)
                }
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
                        .strip_prefix("/mnt/host/source/src/")?
                        .parent()
                        .unwrap();
                    Ok(format!(
                        "//{}:{}",
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
            // The primordial packages need to use the -base SDK because they are used
            // as inputs to the final board SDK.
            format!("{}-base", board)
        } else {
            board.to_owned()
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
        })
    }
}

#[derive(Serialize)]
struct BuildTemplateContext {
    ebuilds: Vec<EBuildEntry>,
}

#[derive(Serialize)]
struct RepositoriesTemplateContext {
    dists: Vec<DistFileEntry>,
    mirrors: &'static [&'static str],
}

static PACKAGE_BUILD_TEMPLATE: &str = include_str!("package-template.BUILD.bazel");

static REPOSITORIES_TEMPLATE: &str = include_str!("repositories-template.bzl");

fn generate_package_build_file(
    board: &str,
    packages_in_dir: &PackagesInDir,
    out: &Path,
    resolver: &PackageResolver,
) -> Result<()> {
    let context = BuildTemplateContext {
        ebuilds: packages_in_dir
            .packages
            .iter()
            .map(|package| EBuildEntry::try_new(board, *package, resolver))
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

fn generate_package(
    board: &str,
    packages_in_dir: PackagesInDir<'_>,
    package_output_dir: &Path,
    translator: &PathTranslator,
    resolver: &PackageResolver,
) -> Result<()> {
    create_dir_all(&packages_in_dir.output_dir)?;

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
    let input_files_dir = packages_in_dir.input_dir.join("files");
    if input_files_dir.try_exists()? {
        let output_files_dir = packages_in_dir.output_dir.join("files");
        symlink(input_files_dir, output_files_dir)?;
    }

    // Create `*.bashrc` symlinks if necessary.
    // TODO: Create symlinks even if there is no ebuild.
    for entry in packages_in_dir.input_dir.read_dir()? {
        let filename = entry?.file_name();
        if filename.to_string_lossy().ends_with(".bashrc") {
            symlink(
                packages_in_dir.input_dir.join(&filename),
                packages_in_dir.output_dir.join(&filename),
            )?;
        }
    }

    // Generate `BUILD.bazel`.
    generate_package_build_file(
        board,
        &packages_in_dir,
        &package_output_dir.join("BUILD.bazel"),
        resolver,
    )
}

fn generate_repositories_file(packages: &Vec<Package>, out: &Path) -> Result<()> {
    let joined_dists: Vec<DistFileEntry> = packages
        .iter()
        .flat_map(|package| {
            package
                .sources
                .remote_sources
                .iter()
                .map(DistFileEntry::try_new)
        })
        .collect::<Result<_>>()?;

    let unique_dists = joined_dists
        .into_iter()
        .sorted_by(|a, b| a.filename.cmp(&b.filename))
        .dedup_by(|a, b| a.filename == b.filename)
        .collect();

    let context = RepositoriesTemplateContext {
        dists: unique_dists,
        mirrors: DEFAULT_MIRRORS,
    };

    let mut templates = TinyTemplate::new();
    templates.add_template("main", REPOSITORIES_TEMPLATE)?;
    let rendered = templates.render("main", &context)?;

    let mut file = File::create(out)?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    file.write_all(rendered.as_bytes())?;
    Ok(())
}

fn evaluate_all_packages(
    repos: &RepositorySet,
    evaluator: &CachedEBuildEvaluator,
) -> Result<Vec<Arc<PackageDetails>>> {
    let ebuild_paths = repos.find_all_ebuilds()?;
    let ebuild_count = ebuild_paths.len();

    let counter = Arc::new(AtomicUsize::new(0));

    // Evaluate packages in parallel.
    let packages = ebuild_paths
        .into_par_iter()
        .map(|ebuild_path| {
            let result = evaluator.evaluate(&ebuild_path);
            let count = 1 + counter.fetch_add(1, Ordering::SeqCst);
            eprint!("Loading ebuilds... {}/{}\r", count, ebuild_count);
            result
        })
        .collect::<Result<Vec<_>>>()?;
    eprintln!("");

    Ok(packages)
}

fn analyze_packages(
    all_details: Vec<Arc<PackageDetails>>,
    resolver: &PackageResolver,
) -> Vec<Package> {
    // Analyze packages in parallel.
    let mut all_packages: Vec<Package> = all_details
        .into_par_iter()
        .flat_map(|details| {
            let result = (|| -> Result<Package> {
                let dependencies = analyze_dependencies(&*details, resolver)?;
                let sources = analyze_sources(&*details)?;
                Ok(Package {
                    details: details.clone(),
                    dependencies,
                    sources,
                })
            })();
            match result {
                Ok(package) => Some(package),
                Err(err) => {
                    println!(
                        "WARNING: {}: Analysis failed: {:#}",
                        details.ebuild_path.to_string_lossy(),
                        err
                    );
                    None
                }
            }
        })
        .collect();

    // Fix-up can be done after analyzing all packages.
    fixup_sources(all_packages.iter_mut().map(|package| &mut package.sources));

    all_packages
}

fn join_by_package_dir<'a>(
    all_packages: &'a Vec<Package>,
    translator: &PathTranslator,
    output_dir: &Path,
) -> HashMap<PathBuf, PackagesInDir<'a>> {
    let mut packages_by_dir = HashMap::<PathBuf, PackagesInDir>::new();

    for package in all_packages.iter() {
        let package_input_dir = translator.translate(package.details.ebuild_path.parent().unwrap());
        let relative_package_dir = match package.details.ebuild_path.strip_prefix(CHROOT_SRC_DIR) {
            Ok(relative_ebuild_path) => relative_ebuild_path.parent().unwrap(),
            Err(_) => continue,
        };
        let package_output_dir = output_dir.join(relative_package_dir);
        packages_by_dir
            .entry(package_output_dir)
            .or_insert_with_key(|package_output_dir| PackagesInDir {
                input_dir: package_input_dir,
                output_dir: package_output_dir.clone(),
                packages: Vec::new(),
            })
            .packages
            .push(package);
    }

    packages_by_dir
}

/// The entry point of "generate-repo" subcommand.
pub fn generate_repo_main(
    board: &str,
    repos: &RepositorySet,
    evaluator: &CachedEBuildEvaluator,
    resolver: &PackageResolver,
    translator: &PathTranslator,
    output_dir: &Path,
) -> Result<()> {
    match remove_dir_all(output_dir) {
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        err => {
            err?;
        }
    };
    create_dir_all(output_dir)?;

    let all_details = evaluate_all_packages(repos, evaluator)?;

    let all_packages = analyze_packages(all_details, resolver);

    let packages_by_dir = join_by_package_dir(&all_packages, translator, output_dir);

    // Generate packages in parallel.
    packages_by_dir
        .into_par_iter()
        .try_for_each(|(relative_output_dir, packages_in_dir)| {
            let package_dir = output_dir.join(relative_output_dir);
            generate_package(board, packages_in_dir, &package_dir, translator, resolver)
        })?;

    generate_repositories_file(&all_packages, &output_dir.join("repositories.bzl"))?;

    File::create(output_dir.join("BUILD.bazel"))?.write_all(&[])?;
    File::create(output_dir.join("WORKSPACE.bazel"))?.write_all(&[])?;

    Ok(())
}
