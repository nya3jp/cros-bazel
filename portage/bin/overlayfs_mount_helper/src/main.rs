// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! A tiny helper to mount overlayfs.

use std::{ffi::OsString, process::ExitCode};

use nix::mount::MsFlags;

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().collect();
    if args.len() != 3 {
        eprintln!("overlayfs_mount_helper: ERROR: wrong number of args");
        return ExitCode::FAILURE;
    }

    let options = &args[1];
    let mount_dir = &args[2];

    match nix::mount::mount(
        Some("overlay"),
        mount_dir.as_os_str(),
        Some("overlay"),
        MsFlags::empty(),
        Some(options.as_os_str()),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!(
                "overlayfs_mount_helper: ERROR: mount failed: {}",
                err.desc()
            );
            ExitCode::FAILURE
        }
    }
}
