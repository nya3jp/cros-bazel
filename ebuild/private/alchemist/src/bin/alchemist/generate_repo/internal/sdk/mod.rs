// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use alchemist::{repository::RepositorySet, toolchain::ToolchainConfig};
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use tera::Tera;

use super::super::common::{AUTOGENERATE_NOTICE, CHROOT_SRC_DIR, CHROOT_THIRD_PARTY_DIR};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template("emerge", include_str!("templates/emerge"))
            .unwrap();
        tera.add_raw_template("make.conf", include_str!("templates/make.conf"))
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
        overlays_targets.push(format!("@//{}", relative.display()));
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
            .filter(|t| t.name == "x86_64-cros-linux-gnu" || t.name == "aarch64-cros-linux-gnu")
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

// TinyTemplate doesn't support hash maps so we make our own K/V pair
// We also want the output sorted correctly.
#[derive(Serialize, Debug)]
struct MakeVar {
    key: String,
    value: String,
}

impl From<(&str, &str)> for MakeVar {
    fn from(tuple: (&str, &str)) -> Self {
        MakeVar {
            key: tuple.0.to_owned(),
            value: tuple.1.to_owned(),
        }
    }
}

impl From<(&str, String)> for MakeVar {
    fn from(tuple: (&str, String)) -> Self {
        MakeVar {
            key: tuple.0.to_owned(),
            value: tuple.1,
        }
    }
}

#[derive(Serialize, Debug)]
struct MakeConfContext {
    sources: Vec<String>,
    vars: Vec<MakeVar>,
}

fn generate_make_conf_board(repos: &RepositorySet, out: &Path) -> Result<()> {
    let mut sources: Vec<String> = Vec::new();
    for repo in repos.get_repos() {
        let make_conf = repo.base_dir().join("make.conf");
        if make_conf.try_exists()? {
            sources.push(
                make_conf
                    .to_str()
                    .context("Invalid make.conf path")?
                    .to_owned(),
            )
        }
    }

    let vars: Vec<MakeVar> = vec![MakeVar::from((
        "USE",
        // TODO(b/265433399): Fix the profiles so we can remove this hack
        "$USE -ondevice_speech",
    ))];

    let context = MakeConfContext { sources, vars };

    let mut file = File::create(out.join("make.conf.board"))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to("make.conf", &tera::Context::from_serialize(context)?, file)?;

    Ok(())
}

fn generate_make_conf_board_setup(
    board: &str,
    repos: &RepositorySet,
    toolchain_config: &ToolchainConfig,
    out: &Path,
) -> Result<()> {
    let vars: Vec<MakeVar> = vec![
        MakeVar::from((
            "ARCH",
            toolchain_config
                .primary()
                .context("No primary toolchain")?
                .portage_arch()?,
        )),
        // BOARD_OVERLAY contains everything that isn't a third_party repo
        MakeVar::from((
            "BOARD_OVERLAY",
            repos
                .get_repos()
                .iter()
                .filter(|r| !r.base_dir().starts_with(CHROOT_THIRD_PARTY_DIR))
                .map(|r| r.base_dir().display())
                .join("\n"),
        )),
        MakeVar::from(("BOARD_USE", board)),
        MakeVar::from((
            "CHOST",
            toolchain_config
                .primary()
                .context("No primary toolchain")?
                .name
                .as_ref(),
        )),
        MakeVar::from((
            "MAKEOPTS",
            // TODO: Read the number of cores in the system
            // Making this dynamic is a problem though because the value gets
            // included in the environment.tgz that's part of the bin pkg. This
            // means we get different outputs when built on different systems.
            // We can't have that. So let's leave it hard coded for now and
            // figure out how to strip it from the environment.tgz.
            "-j128",
        )),
        MakeVar::from(("PKG_CONFIG", format!("/build/{board}/build/bin/pkg-config"))),
        MakeVar::from((
            // TODO: The make.conf actually overrides this variable. Evaluate
            // if we can get rid of it.
            "PORTDIR_OVERLAY",
            repos
                .get_repos()
                .iter()
                .map(|r| r.base_dir().display())
                .join("\n"),
        )),
        MakeVar::from((
            "ROOT",
            // Trailing slash is important!
            format!("/build/{board}/"),
        )),
    ];

    let context = MakeConfContext {
        sources: vec![],
        vars,
    };

    let mut file = File::create(out.join("make.conf.board_setup"))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to("make.conf", &tera::Context::from_serialize(context)?, file)?;

    Ok(())
}

pub fn generate_sdk(
    board: &str,
    repos: &RepositorySet,
    toolchain_config: &ToolchainConfig,
    out: &Path,
) -> Result<()> {
    let out = out.join("internal/sdk");

    create_dir_all(&out)?;

    generate_sdk_build(board, repos, toolchain_config, &out)?;
    if let Some(toolchain) = toolchain_config.primary() {
        generate_wrappers(board, &toolchain.name, &out)?;
    }
    generate_make_conf_board(repos, &out)?;
    generate_make_conf_board_setup(board, repos, toolchain_config, &out)?;

    Ok(())
}
