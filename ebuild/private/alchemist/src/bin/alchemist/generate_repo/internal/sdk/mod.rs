// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;

use alchemist::{
    config::makeconf::generate::generate_make_conf_for_board, fakechroot::PathTranslator,
    repository::RepositorySet, toolchain::ToolchainConfig,
};
use lazy_static::lazy_static;
use serde::Serialize;
use tera::Tera;

use super::super::common::{AUTOGENERATE_NOTICE, CHROOT_SRC_DIR};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template("emerge", include_str!("templates/emerge"))
            .unwrap();
        tera.add_raw_template("pkg-config", include_str!("templates/pkg-config"))
            .unwrap();
        tera.add_raw_template("portage-tool", include_str!("templates/portage-tool"))
            .unwrap();
        tera.add_raw_template("sdk.BUILD.bazel", include_str!("templates/sdk.BUILD.bazel"))
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
    board: &'a str,
    overlays: Vec<String>,
    triples: Vec<&'a str>,
    profile_path: PathBuf,
    wrappers: Vec<&'a str>,
}

fn generate_sdk_build(
    board: &str,
    repos: &RepositorySet,
    toolchain_config: &ToolchainConfig,
    out: &Path,
) -> Result<()> {
    // TODO: Don't hard code the base profile
    let profile_path = repos.primary().base_dir().join("profiles/base");

    let mut overlays_targets: Vec<String> = Vec::new();
    for repo in repos.get_repos() {
        let relative = repo.base_dir().strip_prefix(CHROOT_SRC_DIR)?;
        overlays_targets.push(format!("//internal/overlays/{}", relative.display()));
    }

    let wrappers = WRAPPER_DEFS.iter().map(|def| def.name).collect();

    let context = SdkTemplateContext {
        board,
        triples: toolchain_config
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
        overlays: overlays_targets,
        profile_path,
        wrappers,
    };

    let mut file = File::create(out.join("BUILD.bazel"))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "sdk.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;

    Ok(())
}

pub fn generate_sdk(
    board: &str,
    repos: &RepositorySet,
    toolchain_config: &ToolchainConfig,
    translator: &PathTranslator,
    out: &Path,
) -> Result<()> {
    let out = out.join("internal/sdk");

    create_dir_all(&out)?;

    generate_sdk_build(board, repos, toolchain_config, &out)?;
    if let Some(toolchain) = toolchain_config.primary() {
        generate_wrappers(board, &toolchain.name, &out)?;
    }
    generate_make_conf_for_board(board, repos, toolchain_config, translator, &out)?;

    Ok(())
}
