// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::{create_dir, set_permissions, File, Permissions},
    os::unix::{fs::symlink, prelude::PermissionsExt},
    path::PathBuf,
    process::ExitCode,
};

use anyhow::Result;
use clap::Parser;
use container::{enter_mount_namespace, ContainerSettings};
use durabletree::DurableTree;

#[derive(Copy, Clone, Debug, PartialEq, Eq, strum_macros::Display, strum_macros::EnumString)]
#[strum(serialize_all = "snake_case")]
enum Mode {
    /// Generates a durable tree with a few fixed files.
    Generate,

    /// Checks that the given durable tree can be expanded successfully and
    /// permissions of files are preserved.
    Check,
}

#[derive(Parser, Debug)]
struct Args {
    /// Specifies the operation mode of the helper program.
    mode: Mode,

    /// The directory to read/write a durable tree from/to.
    dir: PathBuf,
}

fn do_main() -> Result<()> {
    let args = Args::try_parse()?;

    let durable_tree_dir = args.dir.as_path();

    match args.mode {
        Mode::Generate => {
            // Create a durable tree with a few files with certain permissions.
            let file = durable_tree_dir.join("file");
            let dir = durable_tree_dir.join("dir");
            let link = durable_tree_dir.join("link");

            File::create(&file)?;
            set_permissions(&file, Permissions::from_mode(0o750))?;
            create_dir(&dir)?;
            set_permissions(&dir, Permissions::from_mode(0o750))?;
            symlink("/path/to/something", link)?;

            DurableTree::convert(durable_tree_dir)?;
        }

        Mode::Check => {
            let durable_tree = DurableTree::expand(durable_tree_dir)?;

            // Mount the durable tree using overlayfs.
            let mut settings = ContainerSettings::new();
            for layer in durable_tree.layers() {
                settings.push_layer(layer)?;
            }
            let container = settings.prepare()?;

            // Inspect contents.
            let file = container.root_dir().join("file");
            let dir = container.root_dir().join("dir");
            let link = container.root_dir().join("link");

            let file_metadata = std::fs::metadata(file)?;
            let file_perm = file_metadata.permissions().mode() & 0o777;
            assert_eq!(
                file_perm, 0o750,
                "Permission mismatch for \"file\": got {:o}, want {:o}",
                file_perm, 0o750
            );

            let dir_metadata = std::fs::metadata(dir)?;
            let dir_perm = dir_metadata.permissions().mode() & 0o777;
            assert_eq!(
                dir_perm, 0o750,
                "Permission mismatch for \"dir\": got {:o}, want {:o}",
                dir_perm, 0o750
            );

            let target = link.read_link()?;
            assert_eq!(
                target.to_string_lossy(),
                "/path/to/something",
                "Symlink target mismatch: got {:?}, want {:?}",
                target.to_string_lossy(),
                "/path/to/something"
            );
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cliutil::cli_main(
        do_main,
        cliutil::ConfigBuilder::new()
            .log_command_line(false)
            .build()
            .unwrap(),
    )
}
