// Copyright 2023 The ChromiumOS Authors
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

use alchemist::{
    analyze::{restrict::analyze_restricts, source::PackageLocalSource},
    dependency::restrict::RestrictAtom,
    ebuild::PackageDetails,
    fakechroot::PathTranslator,
    repository::RepositorySet,
};
use anyhow::{anyhow, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use rayon::prelude::*;
use serde::Serialize;
use tera::Tera;
use tracing::instrument;

use crate::generate_repo::common::{
    AnalysisError, DistFileEntry, Package, AUTOGENERATE_NOTICE, PRIMORDIAL_PACKAGES,
};

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

#[derive(Serialize)]
pub struct EBuildEntry {
    ebuild_name: String,
    basename: String,
    overlay: String,
    category: String,
    version: String,
    sources: Vec<String>,
    git_trees: Vec<String>,
    dists: Vec<DistFileEntry>,
    build_deps: Vec<String>,
    runtime_deps: Vec<String>,
    install_set: Vec<String>,
    allow_network_access: bool,
    uses: String,
    sdk: String,
}

impl EBuildEntry {
    pub fn try_new(target_prefix: &str, package: &Package) -> Result<Self> {
        let ebuild_name = package
            .details
            .ebuild_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let basename = ebuild_name
            .rsplit_once('.')
            .ok_or_else(|| anyhow!("No file extension"))?
            .0
            .to_owned();
        let category = package
            .details
            .package_name
            .split('/')
            .next()
            .expect("Package name must contain a /")
            .to_string();
        let version = package.details.version.to_string();

        let sources = package
            .sources
            .local_sources
            .iter()
            .map(|source| match source {
                PackageLocalSource::BazelTarget(target) => target.clone(),
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

        let format_dependencies =
            |prefix: &str, deps: &[Arc<PackageDetails>]| -> Result<Vec<String>> {
                let targets = deps
                    .iter()
                    .map(|details| {
                        Ok(format!(
                            "//internal/packages/{}/{}/{}:{}",
                            prefix, details.repo_name, details.package_name, details.version
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(targets.into_iter().sorted().dedup().collect())
            };

        let build_deps = format_dependencies(target_prefix, &package.dependencies.build_deps)?;
        let runtime_deps = format_dependencies(target_prefix, &package.dependencies.runtime_deps)?;

        let install_set = format_dependencies(target_prefix, &package.install_set)?;

        let restricts = analyze_restricts(&package.details)?;
        let allow_network_access = restricts.contains(&RestrictAtom::NetworkSandbox);

        let uses = package
            .details
            .use_map
            .iter()
            .sorted_by(|(a_name, a_value), (b_name, b_value)| {
                // Enabled ones comes before disabled ones.
                a_value.cmp(b_value).reverse().then(a_name.cmp(b_name))
            })
            .map(|(name, value)| format!("{}{}", if *value { "" } else { "-" }, name))
            .join(" ");

        let sdk = if PRIMORDIAL_PACKAGES
            .iter()
            .any(|v| v == &package.details.package_name)
        {
            format!("//internal/sdk/{}:base", target_prefix)
        } else {
            format!("//internal/sdk/{}", target_prefix)
        };

        let overlay = format!("//internal/overlays/{}", package.details.repo_name);

        Ok(Self {
            ebuild_name,
            basename,
            overlay,
            category,
            version,
            sources,
            git_trees,
            dists,
            build_deps,
            runtime_deps,
            install_set,
            allow_network_access,
            uses,
            sdk,
        })
    }
}

#[derive(Serialize)]
pub struct EBuildFailure {
    ebuild_name: String,
    version: String,
    error: String,
}

impl EBuildFailure {
    pub fn new(failure: &AnalysisError) -> Self {
        let ebuild_name = failure
            .ebuild
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        EBuildFailure {
            ebuild_name,
            version: failure.version.to_string(),
            error: failure.error.to_string(),
        }
    }
}

#[derive(Serialize)]
struct BuildTemplateContext<'a> {
    board: &'a str,
    overlay_set: &'a str,
    ebuilds: Vec<EBuildEntry>,
    failures: Vec<EBuildFailure>,
}

struct PackagesInDir<'a> {
    packages: Vec<&'a Package>,
    failed_packages: Vec<&'a AnalysisError>,
}

fn generate_package_build_file(
    board: &str,
    overlay_set: &str,
    target_prefix: &str,
    packages_in_dir: &PackagesInDir,
    out: &Path,
) -> Result<()> {
    let context = BuildTemplateContext {
        board,
        overlay_set,
        ebuilds: packages_in_dir
            .packages
            .iter()
            .map(|package| EBuildEntry::try_new(target_prefix, package))
            .collect::<Result<_>>()?,
        failures: packages_in_dir
            .failed_packages
            .iter()
            .map(|failure| EBuildFailure::new(failure))
            .collect(),
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

fn generate_package(
    board: &str,
    overlay_set: &str,
    target_prefix: &str,
    translator: &PathTranslator,
    packages_in_dir: &PackagesInDir,
    output_dir: &Path,
) -> Result<()> {
    create_dir_all(output_dir)?;

    let ebuilds = packages_in_dir
        .packages
        .iter()
        .map(|p| &p.details.ebuild_path)
        .chain(packages_in_dir.failed_packages.iter().map(|f| &f.ebuild));

    // Create `*.ebuild` symlinks.
    for (i, ebuild) in ebuilds.enumerate() {
        let file_name = ebuild.file_name().expect("ebuild must have a file name");
        symlink(translator.to_outer(ebuild)?, output_dir.join(file_name))?;

        if i == 0 {
            // Create a `files` symlink if necessary.
            let files_dir = ebuild.with_file_name("files");
            if files_dir.try_exists()? {
                let output_files_dir = output_dir.join("files");
                symlink(translator.to_outer(files_dir)?, output_files_dir)?;
            }
        }
    }

    generate_package_build_file(
        board,
        overlay_set,
        target_prefix,
        packages_in_dir,
        &output_dir.join("BUILD.bazel"),
    )?;

    Ok(())
}

/// Groups ebuilds into `<repo_name>/<category>/<package>` groups.
fn join_by_package_dir<'p>(
    all_packages: &'p [Package],
    failures: &'p [AnalysisError],
) -> HashMap<PathBuf, PackagesInDir<'p>> {
    let mut packages_by_dir = HashMap::<PathBuf, PackagesInDir>::new();

    let new_default = || PackagesInDir {
        packages: Vec::new(),
        failed_packages: Vec::new(),
    };

    for package in all_packages.iter() {
        packages_by_dir
            .entry(Path::new(&package.details.repo_name).join(&package.details.package_name))
            .or_insert_with(new_default)
            .packages
            .push(package);
    }

    for failure in failures.iter() {
        packages_by_dir
            .entry(Path::new(&failure.repo_name).join(&failure.package_name))
            .or_insert_with(new_default)
            .failed_packages
            .push(failure);
    }

    packages_by_dir
}

#[instrument(skip_all)]
pub fn generate_internal_packages(
    board: &str,
    repo_set: &RepositorySet,
    target_prefix: &str,
    translator: &PathTranslator,
    all_packages: &[Package],
    failures: &[AnalysisError],
    output_dir: &Path,
) -> Result<()> {
    let output_packages_dir = output_dir.join("internal/packages").join(target_prefix);

    let overlay_set = format!("//internal/overlays:{}", repo_set.primary().name());

    // Generate packages in parallel.
    let packages_by_dir = join_by_package_dir(all_packages, failures);
    packages_by_dir
        .into_par_iter()
        .try_for_each(|(relative_package_dir, packages_in_dir)| {
            let output_package_dir = output_packages_dir.join(relative_package_dir);
            generate_package(
                board,
                &overlay_set,
                target_prefix,
                translator,
                &packages_in_dir,
                &output_package_dir,
            )
        })
}
