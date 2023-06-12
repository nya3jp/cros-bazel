// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use binarypackage::BinaryPackage;
use clap::Parser;
use cliutil::cli_main;
use container::enter_mount_namespace;
use durabletree::DurableTree;
use std::{path::PathBuf, process::ExitCode};
use vdb::{create_initial_vdb, generate_vdb_contents, get_vdb_dir};

/// Unpacks a binary package file to generate an installed image that can be
/// mounted as an overlayfs layer.
#[derive(Parser, Debug)]
struct Cli {
    /// Input binary package file.
    #[arg(long)]
    input_binary_package: PathBuf,

    /// Output directory where the installed image is saved as a durable tree.
    #[arg(long)]
    output_directory: PathBuf,

    /// Directory prefix to add to the output image files.
    // Note: This is not `PathBuf` because the default parser doesn't allow
    // empty paths.
    #[arg(long)]
    image_prefix: String,

    /// Directory prefix to add to the output VDB directory.
    // Note: This is not `PathBuf` because the default parser doesn't allow
    // empty paths.
    #[arg(long)]
    vdb_prefix: String,

    /// Indicates that the package is for the host.
    #[arg(long)]
    host: bool,
}

fn do_main() -> Result<()> {
    let args = Cli::parse();

    let image_dir = args.output_directory.join(&args.image_prefix);
    std::fs::create_dir_all(&image_dir)?;

    let mut binary_package = BinaryPackage::open(&args.input_binary_package)?;
    binary_package.extract_image(&image_dir)?;

    let mut contents: Vec<u8> = Vec::new();
    generate_vdb_contents(&mut contents, &image_dir)?;

    let vdb_dir = get_vdb_dir(
        &args.output_directory.join(&args.vdb_prefix),
        binary_package.category_pf(),
    );
    create_initial_vdb(&vdb_dir, &binary_package)?;
    std::fs::write(vdb_dir.join("CONTENTS"), contents)?;

    if args.host {
        // HACK: Rename directories that collide with well-known symlinks.
        //
        // The host profile sets SYMLINK_LIB=yes, which causes sys-libs/glibc to
        // create symlinks /lib -> /lib64 and /usr/lib -> /usr/lib64. Those
        // symlinks are problematic when we use overlayfs to simulate package
        // installation because symlinks are replaced with regular directories
        // when a package contains /lib or /usr/lib as regular directories.
        // Until we set SYMLINK_LIB=no everywhere (crbug.com/360346), we work
        // around the issue by simply renaming directories.
        for (source, target) in [("lib", "lib64"), ("usr/lib", "usr/lib64")] {
            let source = image_dir.join(source);
            let target = image_dir.join(target);
            if source.is_dir() {
                std::fs::rename(&source, &target).with_context(|| {
                    format!(
                        "Failed to rename {} to {}",
                        source.display(),
                        target.display()
                    )
                })?;
            }
        }
    }

    DurableTree::convert(&args.output_directory)?;

    Ok(())
}

fn main() -> ExitCode {
    // We want CAP_DAC_OVERRIDE to scan read-protected directories on generating CONTENTS.
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cli_main(do_main)
}
