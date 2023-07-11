// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use itertools::Itertools;
use std::{
    collections::{BTreeMap, HashMap},
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};
use tracing::instrument;

use alchemist::ebuild::PackageError;
use alchemist::resolver::PackageResolver;
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use rayon::prelude::*;
use serde::Serialize;
use tera::Tera;
use version::Version;

use super::common::{Package, AUTOGENERATE_NOTICE};

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
pub struct AliasEntry {
    name: String,
    actual: String,
}

#[derive(Serialize)]
pub struct TestSuiteEntry {
    name: String,
    test_name: String,
}

#[derive(Serialize)]
struct BuildTemplateContext {
    aliases: Vec<AliasEntry>,
    test_suites: Vec<TestSuiteEntry>,
}

enum MaybePackage<'a> {
    Package(&'a Package),
    PackageError(&'a PackageError),
}

impl MaybePackage<'_> {
    fn repo_name(&self) -> &str {
        match self {
            Self::Package(p) => &p.details.repo_name,
            Self::PackageError(p) => &p.repo_name,
        }
    }
    fn package_name(&self) -> &str {
        match self {
            Self::Package(p) => &p.details.package_name,
            Self::PackageError(p) => &p.package_name,
        }
    }
    fn version(&self) -> &Version {
        match self {
            Self::Package(p) => &p.details.version,
            Self::PackageError(p) => &p.version,
        }
    }
}

fn generate_public_package(
    maybe_packages: Vec<MaybePackage>,
    resolver: &PackageResolver,
    package_output_dir: &Path,
) -> Result<()> {
    create_dir_all(package_output_dir)?;

    let package_details = maybe_packages
        .iter()
        .filter_map(|maybe_package| match maybe_package {
            MaybePackage::Package(p) => Some(p.details.clone()),
            _ => None,
        })
        .collect_vec();

    // Deduplicate versions.
    let version_to_maybe_package: BTreeMap<&Version, &MaybePackage> =
        maybe_packages.iter().map(|p| (p.version(), p)).collect();

    let mut aliases = Vec::new();
    let mut test_suites = Vec::new();

    for (version, maybe_package) in version_to_maybe_package.iter() {
        // TODO(b/278728702): Remove the stage1 hard coded value.
        let internal_package_location = format!(
            "//internal/packages/{}/{}/{}",
            "stage1/target/board",
            maybe_package.repo_name(),
            maybe_package.package_name()
        );
        for suffix in ["", "_debug", "_package_set", "_install"] {
            aliases.push(AliasEntry {
                name: format!("{}{}", version, suffix),
                actual: format!("{}:{}{}", &internal_package_location, version, suffix),
            });
        }
        test_suites.push(TestSuiteEntry {
            name: format!("{}_test", version),
            test_name: format!("{}:{}_test", &internal_package_location, version),
        });
    }

    // Choose the best version to be used for unversioned aliases. Try resolver,
    // with a fallback to a failed package.
    let maybe_best_version = if let Some(best_package) = resolver
        .find_best_package_in(&package_details)
        .with_context(|| format!("Package {:?}", package_details.first()))?
    {
        Some(best_package.version.clone())
    } else {
        maybe_packages
            .iter()
            .filter_map(|maybe_package| match maybe_package {
                MaybePackage::PackageError(p) => Some(p.version.clone()),
                _ => None,
            })
            .last()
    };

    // Generate unversioned aliases.
    if let Some(best_version) = maybe_best_version {
        let short_package_name = &*package_output_dir.file_name().unwrap().to_string_lossy();
        aliases.push(AliasEntry {
            name: short_package_name.to_owned(),
            actual: format!(":{}", &best_version),
        });
        aliases.extend(
            ["debug", "package_set", "install"].map(|suffix| AliasEntry {
                name: suffix.to_owned(),
                actual: format!(":{}_{}", &best_version, suffix),
            }),
        );
        test_suites.push(TestSuiteEntry {
            name: "test".to_owned(),
            test_name: format!(":{}_test", &best_version),
        });
    }

    let context = BuildTemplateContext {
        aliases,
        test_suites,
    };

    let mut file = File::create(package_output_dir.join("BUILD.bazel"))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "package.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;

    Ok(())
}

fn join_by_package_name<'a>(
    all_packages: &'a [Package],
    failed_packages: &'a [PackageError],
) -> HashMap<String, Vec<MaybePackage<'a>>> {
    let mut packages_by_name = HashMap::new();

    let converted_all = all_packages.iter().map(MaybePackage::Package);
    let converted_failed = failed_packages.iter().map(MaybePackage::PackageError);
    for package in converted_all.chain(converted_failed) {
        packages_by_name
            .entry(package.package_name().to_string())
            .or_insert_with(Vec::new)
            .push(package);
    }

    for packages in packages_by_name.values_mut() {
        packages.sort_by(|a, b| a.version().cmp(b.version()));
    }

    packages_by_name
}

#[instrument(skip_all)]
pub fn generate_public_packages(
    all_packages: &[Package],
    failed_packages: &[PackageError],
    resolver: &PackageResolver,
    output_dir: &Path,
) -> Result<()> {
    let packages_by_name = join_by_package_name(all_packages, failed_packages);

    // Generate packages in parallel.
    packages_by_name
        .into_par_iter()
        .try_for_each(|(package_name, maybe_packages)| {
            let package_output_dir = output_dir.join(&package_name);
            generate_public_package(maybe_packages, resolver, &package_output_dir)
        })
}
