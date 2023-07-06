// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::repository::{RepositoryDigest, UnorderedRepositorySet};
use anyhow::Result;
use std::path::PathBuf;

#[derive(clap::Args, Clone, Debug)]
pub struct Args {
    /// Directory used to store a (file_name, mtime) => digest cache.
    cache_dir: Option<String>,

    /// Prints all the files used to calculate the digest
    #[arg(long)]
    print_files: bool,
}

/// The entry point of "digest-repo" subcommand.
pub fn digest_repo_main(repos: &UnorderedRepositorySet, board: &str, args: Args) -> Result<()> {
    // When running inside a cros chroot, files under /etc and /build/$BOARD/etc
    // can also affect the build.
    let additional_dirs_to_digest = vec![
        PathBuf::from("/etc/make.conf"),
        PathBuf::from("/etc/make.conf.board_setup"),
        PathBuf::from("/etc/make.conf.host_setup"),
        PathBuf::from("/etc/make.conf.user"),
        PathBuf::from("/etc/portage"),
        PathBuf::from("/build").join(board).join("etc/make.conf"),
        PathBuf::from("/build")
            .join(board)
            .join("etc/make.conf.board_setup"),
        PathBuf::from("/build")
            .join(board)
            .join("etc/make.conf.user"),
        PathBuf::from("/build").join(board).join("etc/portage"),
    ];

    let digest = RepositoryDigest::new(repos, additional_dirs_to_digest)?;

    if args.print_files {
        for file in digest.file_hashes {
            println!("{:x} {}", file.1, file.0.display());
        }
    }

    println!("{:x}", digest.repo_hash);

    Ok(())
}
