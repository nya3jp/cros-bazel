// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use itertools::Itertools;
use std::{
    borrow::Cow,
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
pub enum SelectValue<'a> {
    Single(Cow<'a, str>),
    // Generates a bazel select() statement with key -> value.
    Select(Vec<(Cow<'a, str>, Cow<'a, str>)>),
}

#[derive(Serialize)]
pub struct AliasEntry<'a> {
    name: Cow<'a, str>,
    actual: SelectValue<'a>,
}

#[derive(Serialize)]
pub struct TestSuiteEntry<'a> {
    name: Cow<'a, str>,
    test_name: Cow<'a, str>,
}

#[derive(Serialize)]
pub struct EbuildFailureEntry<'a> {
    name: Cow<'a, str>,
    error: Cow<'a, str>,
}

#[derive(Serialize)]
struct BuildTemplateContext<'a> {
    aliases: Vec<AliasEntry<'a>>,
    test_suites: Vec<TestSuiteEntry<'a>>,
    ebuild_failures: Vec<EbuildFailureEntry<'a>>,
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

    // Deduplicate versions.
    let version_to_maybe_package: BTreeMap<&Version, &MaybePackage> =
        maybe_packages.iter().map(|p| (p.version(), p)).collect();

    let mut aliases = Vec::new();
    let mut test_suites = Vec::new();

    for (version, maybe_package) in version_to_maybe_package.iter() {
        // TODO(b/278728702): Remove the stage1 hard coded value.
        let internal_package_location_stage1 = format!(
            "//internal/packages/{}/{}/{}",
            "stage1/target/board",
            maybe_package.repo_name(),
            maybe_package.package_name()
        );
        let internal_package_location_stage2 = format!(
            "//internal/packages/{}/{}/{}",
            "stage2/target/board",
            maybe_package.repo_name(),
            maybe_package.package_name()
        );
        for suffix in ["", "_debug", "_package_set", "_install", "_install_list"] {
            aliases.push(AliasEntry {
                name: Cow::from(format!("{}{}", version, suffix)),
                actual: SelectValue::Select(vec![
                    (
                        Cow::Borrowed("//:stage1"),
                        Cow::from(format!(
                            "{}:{}{}",
                            &internal_package_location_stage1, version, suffix
                        )),
                    ),
                    (
                        Cow::Borrowed("//:stage2"),
                        Cow::from(format!(
                            "{}:{}{}",
                            &internal_package_location_stage2, version, suffix
                        )),
                    ),
                ]),
            });
        }
        // The test_suite's tests attribute is not configurable, so we can't
        // use a select. We also can't generate aliases to a test_suite.
        // For now we keep stage1 hard coded until we officially switch over.
        test_suites.push(TestSuiteEntry {
            name: Cow::from(format!("{}_test", version)),
            test_name: Cow::from(format!(
                "{}:{}_test",
                &internal_package_location_stage1, version
            )),
        });
    }

    let package_details = maybe_packages
        .iter()
        .filter_map(|maybe_package| match maybe_package {
            MaybePackage::Package(p) => Some(p.details.clone()),
            _ => None,
        })
        .collect_vec();
    let non_masked_failures = maybe_packages
        .iter()
        .filter_map(|maybe_package| match maybe_package {
            MaybePackage::PackageError(p) => match p.masked {
                Some(true) => None,
                _ => Some(*p),
            },
            _ => None,
        })
        .collect_vec();
    // Choose the best version to be used for unversioned aliases. If there's at
    // least one analysis failure propagate it instead of the normal resolver
    // results (otherwise the build results might be unexpected/incorrect).
    let maybe_best_version = if !non_masked_failures.is_empty() {
        // There are analysis failures.
        None
    } else if let Some(best_package) = resolver
        .find_best_package_in(&package_details)
        .with_context(|| format!("Package {:?}", package_details.first()))?
    {
        Some(best_package.version.clone())
    } else {
        // All packages are masked.
        // TODO(emaxx): Generate ":failure" target with this explanation message.
        None
    };

    // Generate unversioned aliases. In case of failures, all aliases point to
    // the error-printing target.
    let get_actual_target = |suffix: &str| match &maybe_best_version {
        Some(v) => Cow::from(format!(":{}{}", v, suffix)),
        None => Cow::from(":failure"),
    };
    let short_package_name = package_output_dir
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    aliases.push(AliasEntry {
        name: Cow::from(&short_package_name),
        actual: SelectValue::Single(get_actual_target("")),
    });
    for suffix in ["debug", "package_set", "install", "install_list"] {
        if suffix != short_package_name {
            aliases.push(AliasEntry {
                name: Cow::from(suffix),
                actual: SelectValue::Single(get_actual_target(&format!("_{}", suffix))),
            });
        }
        aliases.push(AliasEntry {
            name: Cow::from(format!("{}_{}", short_package_name, suffix)),
            actual: SelectValue::Single(get_actual_target(&format!("_{}", suffix))),
        });
    }
    if short_package_name != "test" {
        test_suites.push(TestSuiteEntry {
            name: Cow::from("test"),
            test_name: get_actual_target("_test"),
        });
    }
    test_suites.push(TestSuiteEntry {
        name: Cow::from(format!("{}_test", short_package_name)),
        test_name: get_actual_target("_test"),
    });

    let ebuild_failures = non_masked_failures
        .iter()
        .map(|failed_package| EbuildFailureEntry {
            name: Cow::from(&failed_package.ebuild_name),
            error: Cow::from(&failed_package.error),
        })
        .collect();

    let context = BuildTemplateContext {
        aliases,
        test_suites,
        ebuild_failures,
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

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: test more than just the syntax and remove this test
    #[test]
    fn template_syntax_valid() -> Result<()> {
        let context = BuildTemplateContext {
            aliases: Vec::new(),
            test_suites: Vec::new(),
            ebuild_failures: Vec::new(),
        };

        let _ = TEMPLATES.render(
            "package.BUILD.bazel",
            &tera::Context::from_serialize(context)?,
        );

        Ok(())
    }
}
