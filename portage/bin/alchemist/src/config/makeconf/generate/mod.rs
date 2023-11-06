// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::borrow::Cow;

use anyhow::{Context, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use tera::Tera;

use crate::{fileops::FileOps, repository::RepositorySet, toolchain::ToolchainConfig};

pub static CHROOT_THIRD_PARTY_DIR: &str = "/mnt/host/source/src/third_party";

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template("make.conf", include_str!("templates/make.conf"))
            .unwrap();
        tera
    };
}

// TinyTemplate doesn't support hash maps so we make our own K/V pair
// We also want the output sorted correctly.
#[derive(Serialize, Debug)]
struct MakeVar<'a> {
    key: Cow<'a, str>,
    value: Cow<'a, str>,
}

impl<'a> From<(&'a str, &'a str)> for MakeVar<'a> {
    fn from(tuple: (&'a str, &'a str)) -> Self {
        MakeVar {
            key: tuple.0.into(),
            value: tuple.1.into(),
        }
    }
}

impl<'a> From<(&'a str, String)> for MakeVar<'a> {
    fn from(tuple: (&'a str, String)) -> Self {
        MakeVar {
            key: tuple.0.into(),
            value: tuple.1.into(),
        }
    }
}

#[derive(Serialize, Debug)]
struct MakeConfContext<'a> {
    sources: Vec<String>,
    vars: Vec<MakeVar<'a>>,
}

fn generate_make_conf_board(repos: &RepositorySet) -> Result<FileOps> {
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

    let vars: Vec<MakeVar> = if repos.get_repo_by_name("chromeos").is_err() {
        // TODO(b/265433399): Fix the profiles so we can remove this hack
        // ondevice_speech binaries are only available for internal builds.
        vec![MakeVar::from(("USE", "$USE -ondevice_speech"))]
    } else {
        vec![]
    };

    let context = MakeConfContext { sources, vars };

    Ok(FileOps::plainfile(
        "/etc/make.conf.board",
        TEMPLATES.render("make.conf", &tera::Context::from_serialize(context)?)?,
    ))
}

fn generate_make_conf_board_setup(
    board: &str,
    repos: &RepositorySet,
    toolchain_config: &ToolchainConfig,
) -> Result<FileOps> {
    let overlays = repos.get_repos().iter().map(|r| r.base_dir()).collect_vec();

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
            overlays
                .iter()
                .filter(|p| !p.starts_with(CHROOT_THIRD_PARTY_DIR))
                .map(|p| p.display())
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
            "-j32",
        )),
        MakeVar::from(("PKG_CONFIG", format!("/build/{board}/build/bin/pkg-config"))),
        MakeVar::from((
            // TODO: The make.conf actually overrides this variable. Evaluate
            // if we can get rid of it.
            "PORTDIR_OVERLAY",
            overlays.iter().map(|p| p.display()).join("\n"),
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

    Ok(FileOps::plainfile(
        "/etc/make.conf.board_setup",
        TEMPLATES.render("make.conf", &tera::Context::from_serialize(context)?)?,
    ))
}

fn generate_make_conf_host_setup() -> Result<FileOps> {
    let vars: Vec<MakeVar> = vec![
        // We need to override the PKGDIR, PORTAGE_TMPDIR, and PORT_LOGDIR
        // that are defined in make.conf.amd64-host because they are pointing
        // to the BROOT. We make our overridden values use $ROOT so that they
        // work when building a new sysroot, and also when building host
        // packages. We could probably upstream this change, but it changes the
        // location of the packages directory from /var/lib/portage/pkgs to
        // /packages.
        MakeVar::from(("PKGDIR", "$ROOT/packages/".to_string())),
        MakeVar::from(("PORTAGE_TMPDIR", "$ROOT/tmp/".to_string())),
        MakeVar::from(("PORT_LOGDIR", "$ROOT/tmp/portage/logs/".to_string())),
    ];

    // TODO:
    // * Add `source /mnt/host/source/src/third_party/chromiumos-overlay/chromeos/config/make.conf.sdk-chromeos`.
    //   Not sure how much this really buys us. I think we should have an amd64-host-private overlay
    //   instead.
    // * Add:
    //       PORTDIR_OVERLAY="$PORTDIR_OVERLAY /mnt/host/source/src/private-overlays/chromeos-partner-overlay"
    //       PORTDIR_OVERLAY="$PORTDIR_OVERLAY /mnt/host/source/src/private-overlays/chromeos-overlay"
    //   Again, I think this should be part of the amd64-host-private overlay
    let context = MakeConfContext {
        sources: vec![],
        vars,
    };

    Ok(FileOps::plainfile(
        "/etc/make.conf.host_setup",
        TEMPLATES.render("make.conf", &tera::Context::from_serialize(context)?)?,
    ))
}

pub fn generate_make_conf_for_board(
    board: &str,
    repos: &RepositorySet,
    toolchain_config: &ToolchainConfig,
) -> Result<Vec<FileOps>> {
    let ops = vec![
        generate_make_conf_board_setup(board, repos, toolchain_config)?,
        if board == "amd64-host" {
            generate_make_conf_host_setup()?
        } else {
            generate_make_conf_board(repos)?
        },
    ];

    Ok(ops)
}
