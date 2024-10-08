// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::{
    analyze::Package,
    dependency::package::{PackageAtom, PackageDependencyAtom},
    ebuild::PackageDetails,
    repository::RepositorySet,
    resolver::PackageResolver,
    toolchain::Toolchain,
};
use anyhow::{bail, Context, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use std::{
    ffi::OsStr,
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};
use std::{str::FromStr, sync::Arc};
use tera::Tera;

use crate::generate_repo::common::{
    package_details_to_target_path, repository_set_to_target_path, PRIMORDIAL_PACKAGES,
    TOOLCHAIN_PACKAGE_NAMES,
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
            "base.BUILD.bazel",
            include_str!("templates/base.BUILD.bazel"),
        )
        .unwrap();
        tera.add_raw_template(
            "host.BUILD.bazel",
            include_str!("templates/host.BUILD.bazel"),
        )
        .unwrap();
        tera.add_raw_template(
            "target.BUILD.bazel",
            include_str!("templates/target.BUILD.bazel"),
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
    primary_triple: Option<&'a str>,
    triples: Vec<&'a str>,
    wrappers: Vec<&'a str>,
    target_deps: Vec<String>,
}

fn get_primordial_packages(resolver: &PackageResolver) -> Result<Vec<Arc<PackageDetails>>> {
    let mut packages = Vec::with_capacity(PRIMORDIAL_PACKAGES.len());
    for package_name in PRIMORDIAL_PACKAGES {
        let atom = PackageDependencyAtom::from_str(package_name)?;

        if resolver.find_provided_packages(&atom).next().is_some() {
            continue;
        }

        let atom = PackageAtom::from_str(package_name)?;
        let best = resolver
            .find_best_package(&atom)?
            .with_context(|| format!("Failed to find {}", package_name))?;

        packages.push(best);
    }

    Ok(packages)
}

fn get_cross_glibc(
    toolchain: &Toolchain,
    resolver: &PackageResolver,
) -> Result<Arc<PackageDetails>> {
    let package_name = format!("cross-{}/glibc", toolchain.name);

    let atom = PackageAtom::from_str(&package_name)?;

    resolver
        .find_best_package(&atom)?
        .with_context(|| format!("Failed to find {}", package_name))
}

fn get_toolchain_packages(
    primary_toolchain: &Toolchain,
    resolver: &PackageResolver,
) -> Result<Vec<Arc<PackageDetails>>> {
    TOOLCHAIN_PACKAGE_NAMES
        .iter()
        .filter(|package_name| {
            if **package_name != "compiler-rt" {
                return true;
            }

            // We only want to emit the compiler-rt package for non-x86
            // platforms since sys-devel/llvm provides the compiler-rt for
            // x86 platforms.
            !primary_toolchain.name.starts_with("x86_64-")
                && !primary_toolchain.name.starts_with("i686-")
        })
        .map(|package_name| {
            let atom = if package_name.contains('/') {
                PackageAtom::from_str(package_name)?
            } else {
                PackageAtom::from_str(&format!(
                    "cross-{}/{}",
                    primary_toolchain.name, package_name
                ))?
            };

            resolver
                .find_best_package(&atom)?
                .with_context(|| format!("Failed to find {}", package_name))
        })
        .collect::<Result<_>>()
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

    /// The packages that will be installed into the SDK.
    pub packages: Vec<&'a Package>,

    /// A suffix to be appended to each package target.
    pub package_suffix: Option<&'a str>,
}

#[derive(Serialize)]
struct SdkBaseContext<'a> {
    name: &'a str,
    overlay_set: &'a str,
    targets: Vec<String>,
    sdk: &'a str,
}

pub fn generate_base_sdk(config: &SdkBaseConfig, out: &Path) -> Result<()> {
    let (dir, target) = match config.name.split_once(':') {
        None => (config.name, config.name),
        Some((_, "")) => bail!("target is blank"),
        Some((dir, target)) => (dir, target),
    };

    let out = out.join("internal/sdk").join(dir);

    create_dir_all(&out)?;

    let context = SdkBaseContext {
        name: target,
        overlay_set: &repository_set_to_target_path(config.source_repo_set),
        targets: config
            .packages
            .iter()
            .map(|package| {
                package_details_to_target_path(&package.details, config.source_package_prefix)
            })
            .map(|mut target| {
                if let Some(suffix) = config.package_suffix {
                    target.push_str(suffix);
                }
                target
            })
            .collect(),
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
}

#[derive(Serialize)]
struct SdkHostContext<'a> {
    name: &'a OsStr,
    base: &'a str,
}

pub fn generate_host_sdk(config: &SdkHostConfig, out: &Path) -> Result<()> {
    let out = out.join("internal/sdk").join(config.name);

    create_dir_all(&out)?;

    let context = SdkHostContext {
        name: Path::new(config.name)
            .file_name()
            .context("Cannot compute name")?,
        base: &format!("//internal/sdk/{}", config.base),
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

pub struct SdkTargetHostConfig<'a> {
    /// The package prefix for host packages.
    ///
    /// i.e., stage2/host
    pub prefix: &'a str,

    /// The host resolver used to lookup toolchain packages.
    pub resolver: &'a PackageResolver,
}

pub struct SdkTargetConfig<'a> {
    /// The base SDK to derive this SDK from.
    pub base: &'a str,

    /// The name of the SDK to generate.
    ///
    /// i.e., stage2/target/board, stage3/target/board, etc
    ///
    /// This is used to generate the path of the SDK.
    /// i.e., //internal/sdk/<name>
    pub name: &'a str,

    /// The name of the target board.
    pub board: &'a str,

    /// Repository set for the target.
    pub target_repo_set: &'a RepositorySet,

    /// Target resolver for looking up primordial packages.
    pub target_resolver: &'a PackageResolver,

    // The primary toolchain used by the target.
    pub target_primary_toolchain: &'a Toolchain,

    // The host config where the cross compiler tools will be sourced. If unset
    // no cross compilers will be added.
    pub host: Option<SdkTargetHostConfig<'a>>,
}

#[derive(Serialize, Debug)]
struct SdkTargetContext<'a> {
    name: &'a str,
    base: &'a str,
    board: &'a str,
    target_overlay_set: &'a str,
    primordial_deps: Vec<String>,
    cross_compiler: Option<SdkTargetCrossCompileContext<'a>>,
    wrappers: Vec<&'a str>,
}

#[derive(Serialize, Debug)]
struct SdkTargetCrossCompileContext<'a> {
    primary_triple: &'a str,
    glibc_target: String,
    toolchain_deps: Vec<String>,
}

pub fn generate_target_sdk(config: &SdkTargetConfig, out: &Path) -> Result<()> {
    let wrappers = WRAPPER_DEFS.iter().map(|def| def.name).collect();

    let out = out.join("internal/sdk").join(config.name);

    create_dir_all(&out)?;

    generate_wrappers(config.board, &config.target_primary_toolchain.name, &out)?;

    let context = SdkTargetContext {
        name: &Path::new(config.name)
            .file_name()
            .context("Cannot compute name")?
            .to_string_lossy(),
        base: &if config.base.starts_with('@') {
            config.base.to_string()
        } else {
            format!("//internal/sdk/{}", config.base)
        },
        board: config.board,
        target_overlay_set: &repository_set_to_target_path(config.target_repo_set),
        primordial_deps: get_primordial_packages(config.target_resolver)?
            .iter()
            .map(|p| package_details_to_target_path(p, config.name))
            .sorted()
            .collect(),
        cross_compiler: match &config.host {
            Some(host) => Some(SdkTargetCrossCompileContext {
                primary_triple: &config.target_primary_toolchain.name,
                glibc_target: package_details_to_target_path(
                    &*get_cross_glibc(config.target_primary_toolchain, host.resolver)?,
                    host.prefix,
                ),
                toolchain_deps: get_toolchain_packages(
                    config.target_primary_toolchain,
                    host.resolver,
                )?
                .iter()
                .map(|p| package_details_to_target_path(p, host.prefix))
                .sorted()
                .collect(),
            }),
            None => None,
        },
        wrappers,
    };

    let mut file = File::create(out.join("BUILD.bazel"))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "target.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;

    Ok(())
}
