// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use clap::Parser;
use cliutil::cli_main;
use container::{enter_mount_namespace, CommonArgs, ContainerSettings};
use fileutil::SafeTempDirBuilder;
use runfiles::Runfiles;

use std::path::PathBuf;
use std::process::{Command, ExitCode};

#[derive(Parser, Debug)]
#[clap()]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,

    /// A path where the tarball is written.
    #[arg(long, required = true)]
    output: PathBuf,
}

fn do_main() -> Result<()> {
    let args = Cli::try_parse()?;

    let r = Runfiles::create()?;

    let fakefs = r.rlocation("cros/bazel/portage/bin/fakefs/fakefs_/fakefs");
    if !fakefs.try_exists()? {
        bail!("{} doesn't exist", fakefs.display());
    }

    let fakefs_preload = r.rlocation("cros/bazel/portage/bin/fakefs/preload/libfakefs_preload.so");
    if !fakefs_preload.try_exists()? {
        bail!("{} doesn't exist", fakefs_preload.display());
    }

    let zstd = r.rlocation("zstd/zstd");
    if !zstd.try_exists()? {
        bail!("{} doesn't exist", zstd.display());
    }

    // Use the parent directory as a tmpdir. /tmp isn't always suitable because
    // it might not be a real filesystem.
    let mutable_base_dir = SafeTempDirBuilder::new()
        .base_dir(args.output.parent().context("output missing parent")?)
        .build()?;

    let mut settings = ContainerSettings::new();
    settings.set_mutable_base_dir(mutable_base_dir.path());
    settings.apply_common_args(&args.common)?;

    let container = settings.prepare()?;

    let root_dir = container.root_dir();

    let mut command = Command::new(fakefs);
    command.arg("--preload");
    command.arg(fakefs_preload);

    command.arg("tar");
    command.arg("--create");
    command.arg(format!("-I{}", zstd.display()));

    command.arg("--file");
    command.arg(&args.output);

    command.arg("-C");
    command.arg(root_dir);

    // Ensure reproducible output.
    command.arg("--format=gnu");
    command.arg("--sort=name");
    command.arg("--mtime=1970-01-01 00:00:00Z");
    command.arg("--numeric-owner");

    // Exclude files and directories crated by the container.
    command.arg("--exclude=.setup.sh");
    command.arg("--exclude=./dev");
    command.arg("--exclude=./host");
    command.arg("--exclude=./proc");
    command.arg("--exclude=./sys");

    command.arg(".");

    command.env("ZSTD_NBTHREADS", "0");
    command.env("ZSTD_CLEVEL", "8");
    // Bazel executes us without a PATH.
    command.env("PATH", "/bin:/usr/bin");

    processes::run_and_check(&mut command)?;

    Ok(())
}

fn main() -> ExitCode {
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cli_main(do_main, Default::default())
}
