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
    #[arg(help = "The runfiles path to the manifest", long)]
    manifest: String,

    #[arg(help = "The directory to install to", long)]
    install_dir: PathBuf,
}

pub fn main() -> Result<()> {
    let args = Cli::parse();
    let r = runfiles::Runfiles::create()?;
    env_logger::init();
    Manifest::create(&r.rlocation(args.manifest))?.install_local(&args.install_dir)
}
