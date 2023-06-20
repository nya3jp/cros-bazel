// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::repository::RepositorySet;
use alchemist::{dependency::package::PackageAtom, ebuild::PackageDetails};
use std::ffi::OsStr;
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};
use std::{str::FromStr, sync::Arc};

use anyhow::{Context, Result};

use alchemist::{
    config::makeconf::generate::generate_make_conf_for_board, fakechroot::PathTranslator,
    resolver::PackageResolver,
};
use lazy_static::lazy_static;
use serde::Serialize;
use tera::Tera;
use tracing::instrument;

use crate::{
    alchemist::TargetData,
    generate_repo::common::{
        package_details_to_package_set_target_path, repository_set_to_target_path,
        PRIMORDIAL_PACKAGES,
    },
    generate_repo::Package,
};

use super::super::common::AUTOGENERATE_NOTICE;

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template("emerge", include_str!("templates/emerge"))
            .unwrap();
        tera.add_raw_template("pkg-config", include_str!("templates/pkg-config"))
            .unwrap();
        tera.add_raw_template("portage-tool", include_str!("templates/portage-tool"))
            .unwrap();
        tera.add_raw_template(
            "stage1.BUILD.bazel",
            include_str!("templates/stage1.BUILD.bazel"),
        )
        .unwrap();
        tera.add_raw_template(
            "base.BUILD.bazel",
            include_str!("templates/base.BUILD.bazel"),
        )
        .unwrap();
        tera.add_raw_template(
            "host.BUILD.bazel",
            include_str!("templates/host.BUILD.bazel"),
        )
        .unwrap();
        tera
    };
}

struct WrapperDef {
    name: &'static str,
    template: &'static str,
}

const WRAPPER_DEFS: &[WrapperDef] = &[
    WrapperDef {
        name: "pkg-config",
        template: "pkg-config",
    },
    WrapperDef {
        name: "emerge",
        template: "emerge",
    },
    WrapperDef {
        name: "ebuild",
        template: "portage-tool",
    },
    WrapperDef {
        name: "eclean",
        template: "portage-tool",
    },
    WrapperDef {
        name: "emaint",
        template: "portage-tool",
    },
    WrapperDef {
        name: "equery",
        template: "portage-tool",
    },
    WrapperDef {
        name: "portageq",
        template: "portage-tool",
    },
    WrapperDef {
        name: "qcheck",
        template: "portage-tool",
    },
    WrapperDef {
        name: "qdepends",
        template: "portage-tool",
    },
    WrapperDef {
        name: "qfile",
        template: "portage-tool",
    },
    WrapperDef {
        name: "qlist",
        template: "portage-tool",
    },
    WrapperDef {
        name: "qmerge",
        template: "portage-tool",
    },
    WrapperDef {
        name: "qsize",
        template: "portage-tool",
    },
];

#[derive(Serialize, Debug)]
struct WrapperContext<'a> {
    name: &'a str,
    board: &'a str,
    triple: &'a str,
}

fn generate_wrappers(board: &str, triple: &str, out: &Path) -> Result<()> {
    for def in WRAPPER_DEFS.iter() {
        let context = WrapperContext {
            name: def.name,
            board,
            triple,
        };

        let file = File::create(out.join(def.name))?;
        TEMPLATES.render_to(def.template, &tera::Context::from_serialize(context)?, file)?;
    }

    Ok(())
}

#[derive(Serialize, Debug)]
struct SdkTemplateContext<'a> {
    name: &'a str,
    board: &'a str,
    overlay_set: &'a str,
    triples: Vec<&'a str>,
    profile_path: PathBuf,
    wrappers: Vec<&'a str>,
    target_deps: Vec<String>,
}

fn get_primordial_packages(resolver: &PackageResolver) -> Result<Vec<Arc<PackageDetails>>> {
    let mut packages = Vec::with_capacity(PRIMORDIAL_PACKAGES.len());
    for package_name in PRIMORDIAL_PACKAGES {
        let atom = PackageAtom::from_str(package_name)?;
        let matches = resolver.find_packages(&atom)?;
        let best = resolver
            .find_best_package_in(&matches)?
            .with_context(|| format!("Failed to find {}", package_name))?;

        if !resolver.is_provided(&best.package_name, &best.version) {
            packages.push(best);
        }
    }

    Ok(packages)
}

fn profile_path(repos: &RepositorySet, profile: &str) -> PathBuf {
    repos.primary().base_dir().join("profiles").join(profile)
}

fn generate_sdk_build(prefix: &str, target: &TargetData, out: &Path) -> Result<()> {
    let wrappers = WRAPPER_DEFS.iter().map(|def| def.name).collect();

    let context = SdkTemplateContext {
        name: &Path::new(prefix)
            .file_name()
            .with_context(|| format!("Invalid prefix: {prefix}"))?
            .to_string_lossy(),
        board: &target.board,
        overlay_set: &repository_set_to_target_path(&target.repos),
        triples: target
            .toolchains
            .toolchains
            .iter()
            // TODO: We only have the prebuilds for the following two
            // toolchains defined. Add the rest of the prebuilds and then
            // remove this.
            .filter(|t| {
                t.name == "x86_64-cros-linux-gnu"
                    || t.name == "aarch64-cros-linux-gnu"
                    || t.name == "armv7a-cros-linux-gnueabihf"
            })
            .map(|t| t.name.as_ref())
            .collect(),
        profile_path: profile_path(&target.repos, &target.profile),
        wrappers,
        target_deps: get_primordial_packages(&target.resolver)?
            .iter()
            .map(|p| {
                format!(
                    "//internal/packages/{}/{}/{}:{}",
                    prefix, p.repo_name, p.package_name, p.version
                )
            })
            .collect(),
    };

    let mut file = File::create(out.join("BUILD.bazel"))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "stage1.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;

    Ok(())
}

#[instrument(skip_all)]
pub fn generate_stage1_sdk(
    prefix: &str,
    target: &TargetData,
    translator: &PathTranslator,
    out: &Path,
) -> Result<()> {
    let out = out.join("internal/sdk").join(prefix);

    create_dir_all(&out)?;

    generate_sdk_build(prefix, target, &out)?;
    if let Some(toolchain) = target.toolchains.primary() {
        generate_wrappers(&target.board, &toolchain.name, &out)?;
    }
    generate_make_conf_for_board(
        &target.board,
        &target.repos,
        &target.toolchains,
        translator,
        &out,
    )?;

    Ok(())
}

pub struct SdkBaseConfig<'a> {
    /// The name of the SDK to generate.
    ///
    /// i.e., stage2, stage3, etc
    ///
    /// This is used to generate the path of the SDK.
    /// i.e., //internal/sdk/<name>
    pub name: &'a str,

    /// The prefix of the packages that will be bundled into the SDK.
    pub source_package_prefix: &'a str,

    /// The SDK that was used to generate the source packages.
    pub source_sdk: &'a str,

    /// Repository set for the host.
    pub source_repo_set: &'a RepositorySet,

    /// The `virtual` package that lists all the runtime dependencies that
    /// will be installed into the SDK.
    pub bootstrap_package: &'a Package,
}

#[derive(Serialize)]
struct SdkBaseContext<'a> {
    name: &'a str,
    overlay_set: &'a str,
    target: &'a str,
    sdk: &'a str,
}

pub fn generate_base_sdk(config: &SdkBaseConfig, out: &Path) -> Result<()> {
    let out = out.join("internal/sdk").join(config.name);

    create_dir_all(&out)?;

    let context = SdkBaseContext {
        name: config.name,
        overlay_set: &repository_set_to_target_path(config.source_repo_set),
        target: &package_details_to_package_set_target_path(
            &config.bootstrap_package.details,
            config.source_package_prefix,
        ),
        sdk: &format!("//internal/sdk/{}", config.source_sdk),
    };

    let mut file = File::create(out.join("BUILD.bazel"))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "base.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;

    Ok(())
}

pub struct SdkHostConfig<'a> {
    /// The base SDK to derive this SDK from.
    pub base: &'a str,

    /// The name of the SDK to generate.
    ///
    /// i.e., stage2/host, stage3/host, etc
    ///
    /// This is used to generate the path of the SDK.
    /// i.e., //internal/sdk/<name>
    pub name: &'a str,

    /// Repository set for the host.
    pub repo_set: &'a RepositorySet,

    // The profile used to build packages.
    pub profile: &'a str,
}

#[derive(Serialize)]
struct SdkHostContext<'a> {
    name: &'a OsStr,
    base: &'a str,
    overlay_set: &'a str,
    profile_path: &'a Path,
}

pub fn generate_host_sdk(config: &SdkHostConfig, out: &Path) -> Result<()> {
    let out = out.join("internal/sdk").join(config.name);

    create_dir_all(&out)?;

    let context = SdkHostContext {
        name: Path::new(config.name)
            .file_name()
            .context("Cannot compute name")?,
        base: &format!("//internal/sdk/{}", config.base),
        overlay_set: &repository_set_to_target_path(config.repo_set),
        profile_path: &profile_path(&config.repo_set, &config.profile),
    };

    let mut file = File::create(out.join("BUILD.bazel"))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "host.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;

    Ok(())
}
