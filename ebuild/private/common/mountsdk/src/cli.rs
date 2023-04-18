// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::mountsdk::{Config, LoginMode};
use anyhow::Result;
use clap::{arg, Args};
use std::collections::HashMap;
use std::path::PathBuf;

pub const SOURCE_DIR: &str = "/mnt/host/source";

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[arg(long, required = true)]
    board: String,

    #[arg(
        long,
        help = "mounts a file or directory as a file system layer in the container.",
        required = true
    )]
    layer: Vec<PathBuf>,

    #[arg(
        long,
        help = "enables the runfiles mode in which input paths are handled as runfile paths."
    )]
    runfiles_mode: bool,

    #[arg(
        long = "login",
        help = "logs in to the SDK before installing deps, before building, after \
            building, or after failing to build respectively.",
        default_value_t = LoginMode::Never,
    )]
    login_mode: LoginMode,
}

impl ConfigArgs {
    pub fn runfiles_mode(&self) -> bool {
        self.runfiles_mode
    }
}

impl Config {
    pub fn try_from(args: ConfigArgs) -> Result<Config> {
        let layer_paths = if args.runfiles_mode {
            let r = runfiles::Runfiles::create()?;
            args.layer
                .into_iter()
                .map(|layer| r.rlocation(layer))
                .collect()
        } else {
            args.layer
        };
        Ok(Config {
            board: args.board,
            layer_paths,
            login_mode: args.login_mode,
            allow_network_access: false,
            privileged: false,
            bind_mounts: Vec::new(),
            envs: HashMap::new(),
        })
    }
}
