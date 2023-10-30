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

use alchemist::{ebuild::MaybePackageDetails, resolver::PackageResolver};
use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use rayon::prelude::*;
use serde::Serialize;
use tera::Tera;
use version::Version;

use super::common::{MaybePackage, AUTOGENERATE_NOTICE};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template(
            "images.BUILD.bazel",
            include_str!("templates/images.BUILD.bazel"),
        )
        .unwrap();
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

fn get_ebuild_name_from_path(ebuild_path: &Path) -> Result<String> {
    Ok(ebuild_path
        .file_name()
        .ok_or_else(|| anyhow!("Failed to extract file name from ebuild path {ebuild_path:?}"))?
        .to_string_lossy()
        .to_string())
}

fn generate_public_package(
    maybe_packages: &[&MaybePackage],
    resolver: &PackageResolver,
    targets: &[TargetConfig],
    test_prefix: &str,
    package_output_dir: &Path,
) -> Result<()> {
    create_dir_all(package_output_dir)?;

    // Deduplicate versions.
    let version_to_maybe_package: BTreeMap<&Version, &MaybePackage> = maybe_packages
        .iter()
        .map(|p| (&p.as_basic_data().version, *p))
        .collect();

    let mut aliases = Vec::new();
    let mut test_suites = Vec::new();

    for (version, maybe_package) in version_to_maybe_package.iter() {
        for (suffix, target_suffix) in [
            ("", "_including_provided"),
            ("_debug", ""),
            ("_package_set", ""),
            ("_install", ""),
            ("_install_list", ""),
        ] {
            aliases.push(AliasEntry {
                name: Cow::from(format!("{}{}", version, suffix)),
                actual: SelectValue::Select(
                    targets
                        .iter()
                        .map(|target| {
                            (
                                Cow::from(format!("@//bazel/portage:{}", target.config)),
                                Cow::from(format!(
                                    "//internal/packages/{}/{}/{}:{}{}{}",
                                    target.prefix,
                                    maybe_package.as_basic_data().repo_name,
                                    maybe_package.as_basic_data().package_name,
                                    version,
                                    suffix,
                                    target_suffix,
                                )),
                            )
                        })
                        .collect(),
                ),
            });
        }
        // The test_suite's tests attribute is not configurable, so we can't
        // use a select. We also can't generate aliases to a test_suite.
        // For now we keep stage1 hard coded until we officially switch over.
        test_suites.push(TestSuiteEntry {
            name: Cow::from(format!("{}_test", version)),
            test_name: Cow::from(format!(
                "//internal/packages/{}/{}/{}:{}_test",
                test_prefix,
                maybe_package.as_basic_data().repo_name,
                maybe_package.as_basic_data().package_name,
                version,
            )),
        });
    }

    let package_details = maybe_packages
        .iter()
        .filter_map(|maybe_package| match maybe_package {
            MaybePackage::Ok(package) => Some(package.details.clone()),
            _ => None,
        })
        .collect_vec();
    let non_masked_failures = maybe_packages
        .iter()
        .filter_map(|maybe_package| match maybe_package {
            MaybePackage::Err(error) => match &error.details {
                MaybePackageDetails::Ok(details) if details.masked => None,
                _ => Some(error.as_ref()),
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
        Some(best_package.as_basic_data().version.clone())
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
            name: Cow::from(
                get_ebuild_name_from_path(&failed_package.as_basic_data().ebuild_path).unwrap(),
            ),
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

fn join_by_package_name(all_packages: &[MaybePackage]) -> HashMap<String, Vec<&MaybePackage>> {
    let mut packages_by_name: HashMap<String, Vec<&MaybePackage>> = HashMap::new();
    for package in all_packages {
        packages_by_name
            .entry(package.as_basic_data().package_name.clone())
            .or_default()
            .push(package);
    }
    packages_by_name
}

#[derive(Debug)]
pub struct TargetConfig<'a> {
    /// The //bazel/portage:<config> setting used to select this target.
    pub config: &'a str,
    /// Package prefix to use when constructing the full target path.
    pub prefix: &'a str,
}

/// Generates the public aliases for packages.
///
/// # Arguments
///
/// * test_prefix: The package prefix to use for testing targets. We can't use
///   a switch statement with `test_suite`, so we can only define one stage
///   to run tests for.
#[instrument(skip_all)]
pub fn generate_public_packages(
    all_packages: &[MaybePackage],
    resolver: &PackageResolver,
    targets: &[TargetConfig],
    test_prefix: &str,
    output_dir: &Path,
) -> Result<()> {
    let packages_by_name = join_by_package_name(all_packages);

    // Generate packages in parallel.
    packages_by_name
        .into_par_iter()
        .try_for_each(|(package_name, maybe_packages)| {
            let package_output_dir = output_dir.join(package_name);
            generate_public_package(
                &maybe_packages,
                resolver,
                targets,
                test_prefix,
                &package_output_dir,
            )
        })
}

#[derive(Serialize)]
struct ImagesTemplateContext<'a> {
    board: &'a str,
}

/// Generates the public targets for images.
#[instrument(skip_all)]
pub fn generate_public_images(board: &str, output_dir: &Path) -> Result<()> {
    create_dir_all(output_dir)?;

    let context = ImagesTemplateContext { board };

    let mut file = File::create(output_dir.join("BUILD.bazel"))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "images.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;
    Ok(())
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
