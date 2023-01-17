// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};

use anyhow::Result;
use rayon::prelude::*;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use super::common::{Package, AUTOGENERATE_NOTICE, CHROOT_SRC_DIR};

static PACKAGE_BUILD_TEMPLATE: &str = include_str!("package-template.BUILD.bazel");

#[derive(Serialize)]
pub struct AliasEntry {
    name: String,
    actual: String,
}

#[derive(Serialize)]
struct BuildTemplateContext {
    aliases: Vec<AliasEntry>,
}

fn generate_public_package(packages: Vec<&Package>, package_output_dir: &Path) -> Result<()> {
    create_dir_all(package_output_dir)?;

    let context = BuildTemplateContext {
        aliases: packages
            .into_iter()
            .flat_map(|package| {
                let details = &package.details;
                let package_relative_dir = match details.ebuild_path.strip_prefix(CHROOT_SRC_DIR) {
                    Ok(ebuild_relative_path) => ebuild_relative_path.parent().unwrap(),
                    _ => return Vec::new(),
                };
                let internal_package_location = format!(
                    "//internal/overlays/{}",
                    package_relative_dir.to_string_lossy(),
                );
                vec![
                    AliasEntry {
                        name: details.version.to_string(),
                        actual: format!(
                            "{}:{}",
                            &internal_package_location,
                            details.version.to_string()
                        ),
                    },
                    AliasEntry {
                        name: format!("{}_package_set", details.version.to_string()),
                        actual: format!(
                            "{}:{}_package_set",
                            &internal_package_location,
                            details.version.to_string()
                        ),
                    },
                ]
            })
            .collect(),
    };

    let mut templates = TinyTemplate::new();
    templates.add_template("main", PACKAGE_BUILD_TEMPLATE)?;
    let rendered = templates.render("main", &context)?;

    let mut file = File::create(package_output_dir.join("BUILD.bazel"))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    file.write_all(rendered.as_bytes())?;
    Ok(())
}

fn join_by_package_name(all_packages: &Vec<Package>) -> HashMap<String, Vec<&Package>> {
    let mut packages_by_name = HashMap::<String, Vec<&Package>>::new();

    for package in all_packages.iter() {
        packages_by_name
            .entry(package.details.package_name.clone())
            .or_insert_with(|| Vec::new())
            .push(package);
    }

    for packages in packages_by_name.values_mut() {
        packages.sort_by(|a, b| a.details.version.cmp(&b.details.version));
    }

    packages_by_name
}

pub fn generate_public_packages(all_packages: &Vec<Package>, output_dir: &Path) -> Result<()> {
    let packages_by_name = join_by_package_name(&all_packages);

    // Generate packages in parallel.
    packages_by_name
        .into_par_iter()
        .try_for_each(|(package_name, packages)| {
            let package_output_dir = output_dir.join(package_name);
            generate_public_package(packages, &package_output_dir)
        })
}
