// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{ensure, Result};
use clap::Parser;
use cliutil::cli_main;
use container::{enter_mount_namespace, BindMount, CommonArgs, ContainerSettings};
use fileutil::resolve_symlink_forest;
use std::{path::PathBuf, process::ExitCode};

const MAIN_SCRIPT: &str = "/mnt/host/.generate_reclient_inputs/generate_reclient_inputs.sh";

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,

    /// The path of output file.
    /// A .tar.zst suffix is expected
    #[arg(long, required = true)]
    output: PathBuf,
}

fn do_main() -> Result<()> {
    let args = Cli::try_parse()?;

    let mut settings = ContainerSettings::new();
    settings.apply_common_args(&args.common)?;

    let runfiles = runfiles::Runfiles::create()?;

    settings.push_bind_mount(BindMount {
        source: resolve_symlink_forest(&runfiles.rlocation(
            "cros/bazel/portage/bin/generate_reclient_inputs/generate_reclient_inputs.sh",
        ))?,
        mount_path: PathBuf::from(MAIN_SCRIPT),
        rw: false,
    });

    // Create the output file, then drop the reference to close the handle.
    // We need the file to exist so the bind mount will work.
    drop(std::fs::File::create(&args.output)?);

    // We want the container to directly write to the output file to avoid
    // copying the tarball from /tmp to the output root.
    settings.push_bind_mount(BindMount {
        source: args.output,
        mount_path: PathBuf::from("/mnt/host/.generate_reclient_inputs/output.tar.zst"),
        rw: true,
    });

    let mut container = settings.prepare()?;
    let mut command = container.command(MAIN_SCRIPT);
    let status = command.status()?;
    ensure!(status.success(), "Command failed: {:?}", status);

    Ok(())
}

fn main() -> ExitCode {
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cli_main(do_main, Default::default())
}
