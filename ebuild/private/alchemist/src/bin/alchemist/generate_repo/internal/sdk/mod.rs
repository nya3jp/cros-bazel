// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::{
    dependency::package::PackageAtom, ebuild::PackageDetails, repository::RepositorySet,
    toolchain::Toolchain,
};
use itertools::Itertools;
use std::{
    ffi::OsStr,
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
        package_details_to_package_set_target_path, package_details_to_target_path,
        repository_set_to_target_path, Package, PRIMORDIAL_PACKAGES, TOOLCHAIN_PACKAGE_NAMES,
    },
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
    profile_path: PathBuf,
    wrappers: Vec<&'a str>,
    target_deps: Vec<String>,
}

fn get_primordial_packages(resolver: &PackageResolver) -> Result<Vec<Arc<PackageDetails>>> {
    let mut packages = Vec::with_capacity(PRIMORDIAL_PACKAGES.len());
    for package_name in PRIMORDIAL_PACKAGES {
        let atom = PackageAtom::from_str(package_name)?;
        let best = resolver
            .find_best_package(&atom)?
            .with_context(|| format!("Failed to find {}", package_name))?;

        if !resolver.is_provided(&best.package_name, &best.version) {
            packages.push(best);
        }
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
        primary_triple: target.toolchains.primary().map(|t| t.name.as_str()),
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

    /// The package prefix for host packages.
    ///
    /// i.e., stage2/host
    pub host_prefix: &'a str,

    /// The host resolver used to lookup toolchain packages.
    pub host_resolver: &'a PackageResolver,

    /// The name of the target board.
    pub board: &'a str,

    /// Repository set for the target.
    pub target_repo_set: &'a RepositorySet,

    /// Target resolver for looking up primordial packages.
    pub target_resolver: &'a PackageResolver,

    /// Target toolchain that will be used to generate the cross-compiler
    /// layer. If None, then no cross-compiler tools will be included.
    pub target_primary_toolchain: Option<&'a Toolchain>,
}

#[derive(Serialize, Debug)]
struct SdkTargetContext<'a> {
    name: &'a str,
    base: &'a str,
    board: &'a str,
    target_overlay_set: &'a str,
    primordial_deps: Vec<String>,
    cross_compiler: Option<SdkTargetCrossCompileContext<'a>>,
}

#[derive(Serialize, Debug)]
struct SdkTargetCrossCompileContext<'a> {
    primary_triple: &'a str,
    glibc_target: String,
    toolchain_deps: Vec<String>,
}

pub fn generate_target_sdk(config: &SdkTargetConfig, out: &Path) -> Result<()> {
    let out = out.join("internal/sdk").join(config.name);

    create_dir_all(&out)?;

    let context = SdkTargetContext {
        name: &Path::new(config.name)
            .file_name()
            .context("Cannot compute name")?
            .to_string_lossy(),
        base: &format!("//internal/sdk/{}", config.base),
        board: config.board,
        target_overlay_set: &repository_set_to_target_path(config.target_repo_set),
        primordial_deps: get_primordial_packages(config.target_resolver)?
            .iter()
            .map(|p| package_details_to_target_path(p, config.name))
            .sorted()
            .collect(),
        cross_compiler: match config.target_primary_toolchain {
            Some(target_primary_toolchain) => Some(SdkTargetCrossCompileContext {
                primary_triple: &target_primary_toolchain.name,
                glibc_target: package_details_to_target_path(
                    &*get_cross_glibc(target_primary_toolchain, config.host_resolver)?,
                    config.host_prefix,
                ),
                toolchain_deps: get_toolchain_packages(
                    target_primary_toolchain,
                    config.host_resolver,
                )?
                .iter()
                .map(|p| package_details_to_target_path(p, config.host_prefix))
                .sorted()
                .collect(),
            }),
            None => None,
        },
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
