// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use clap::Parser;
use manifest::Manifest;
use std::path::PathBuf;

#[derive(Parser)]
#[clap()]
struct Cli {
    /// The runfiles path to the manifest
    #[arg(long)]
    manifest: String,

    /// The directory to install to
    #[arg(long)]
    install_dir: PathBuf,
}

pub fn main() -> Result<()> {
    let args = Cli::try_parse()?;
    let r = runfiles::Runfiles::create()?;
    env_logger::init();
    Manifest::create(&runfiles::rlocation!(r, args.manifest))?.install_local(&args.install_dir)
}
