// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! A tiny helper to mount overlayfs.

use std::{
    ffi::{OsStr, OsString},
    process::ExitCode,
};

use nix::mount::MsFlags;

fn mount_overlayfs(mount_dir: &OsStr, options: &OsStr) -> nix::Result<()> {
    nix::mount::mount(
        Some("overlay"),
        mount_dir,
        Some("overlay"),
        MsFlags::empty(),
        Some(options),
    )
}

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().collect();
    if args.len() != 3 {
        eprintln!("overlayfs_mount_helper: ERROR: wrong number of args");
        return ExitCode::FAILURE;
    }

    let options = &args[1];
    let mount_dir = &args[2];

    // Ubuntu has applied their own patch to the kernel which changes the default behavior of
    // overlayfs. If there is no "userxattr" or "nouserxattr" in `options`, add "nouserxattr" to
    // make it consistent with the normal Linux kernel's default behavior. b/296450672
    if options
        .to_string_lossy()
        .split(',')
        .all(|option| option != "userxattr" && option != "nouserxattr")
    {
        assert!(!options.is_empty());
        let modified_options = String::from(options.to_string_lossy()) + ",nouserxattr";
        match mount_overlayfs(mount_dir, &OsString::from(modified_options)) {
            Ok(()) => {
                return ExitCode::SUCCESS;
            }
            Err(err) => {
                // EINVAL may mean that this is running on a Linux distribution other than Ubuntu
                // that doesn't support "nouserxattr" as an overlayfs mount option.
                // In that case, ignore the error and contniue mounting without "nouserxattr"
                if err != nix::errno::Errno::EINVAL {
                    eprintln!(
                        "overlayfs_mount_helper: ERROR: mount failed: {}",
                        err.desc()
                    );
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    match mount_overlayfs(mount_dir, options) {
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
