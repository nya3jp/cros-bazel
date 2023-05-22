// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::{dependency::package::PackageAtom, ebuild::PackageDetails};
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

use crate::{alchemist::TargetData, generate_repo::common::PRIMORDIAL_PACKAGES};

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
        packages.push(best);
    }

    Ok(packages)
}

fn generate_sdk_build(prefix: &str, target: &TargetData, out: &Path) -> Result<()> {
    let profile_path = target
        .repos
        .primary()
        .base_dir()
        .join("profiles")
        .join(&target.profile);

    let wrappers = WRAPPER_DEFS.iter().map(|def| def.name).collect();

    let context = SdkTemplateContext {
        name: &Path::new(prefix)
            .file_name()
            .with_context(|| format!("Invalid prefix: {prefix}"))?
            .to_string_lossy(),
        board: &target.board,
        overlay_set: &format!("//internal/overlays:{}", target.repos.primary().name()),
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
        profile_path,
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
