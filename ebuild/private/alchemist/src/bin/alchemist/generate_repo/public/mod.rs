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

fn generate_public_package(
    packages: Vec<&Package>,
    resolver: &PackageResolver,
    package_output_dir: &Path,
) -> Result<()> {
    create_dir_all(package_output_dir)?;

    let package_details = packages
        .iter()
        .map(|package| package.details.clone())
        .collect_vec();

    // Deduplicate versions.
    let version_to_package: BTreeMap<Version, &Package> = packages
        .into_iter()
        .map(|package| (package.details.version.clone(), package))
        .collect();

    let mut aliases = Vec::new();
    let mut test_suites = Vec::new();

    for package in version_to_package.values() {
        let details = &package.details;
        let internal_package_location = format!(
            "//internal/packages/{}/{}",
            details.repo_name, details.package_name
        );
        for suffix in ["", "_debug", "_package_set"] {
            aliases.push(AliasEntry {
                name: format!("{}{}", details.version, suffix),
                actual: format!(
                    "{}:{}{}",
                    &internal_package_location, details.version, suffix
                ),
            });
        }
        test_suites.push(TestSuiteEntry {
            name: format!("{}_test", details.version),
            test_name: format!("{}:{}_test", &internal_package_location, details.version),
        });
    }

    if let Some(best_package) = resolver
        .find_best_package_in(&package_details)
        .with_context(|| format!("Package {:?}", package_details.first()))?
    {
        let best_version = &best_package.version;
        let short_package_name = &*package_output_dir.file_name().unwrap().to_string_lossy();
        aliases.push(AliasEntry {
            name: short_package_name.to_owned(),
            actual: format!(":{}", &best_version),
        });
        aliases.extend(["debug", "package_set"].map(|suffix| AliasEntry {
            name: suffix.to_owned(),
            actual: format!(":{}_{}", &best_version, suffix),
        }));
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

fn join_by_package_name(all_packages: &[Package]) -> HashMap<String, Vec<&Package>> {
    let mut packages_by_name = HashMap::<String, Vec<&Package>>::new();

    for package in all_packages.iter() {
        packages_by_name
            .entry(package.details.package_name.clone())
            .or_insert_with(Vec::new)
            .push(package);
    }

    for packages in packages_by_name.values_mut() {
        packages.sort_by(|a, b| a.details.version.cmp(&b.details.version));
    }

    packages_by_name
}

pub fn generate_public_packages(
    all_packages: &[Package],
    resolver: &PackageResolver,
    output_dir: &Path,
) -> Result<()> {
    let packages_by_name = join_by_package_name(all_packages);

    // Generate packages in parallel.
    packages_by_name
        .into_par_iter()
        .try_for_each(|(package_name, packages)| {
            let package_output_dir = output_dir.join(package_name);
            generate_public_package(packages, resolver, &package_output_dir)
        })
}
