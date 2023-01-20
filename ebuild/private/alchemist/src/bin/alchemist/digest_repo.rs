// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alchemist::repository::RepositoryLookup;
use anyhow::Result;
use std::path::Path;

#[derive(clap::Args, Clone, Debug)]
pub struct Args {
    /// Directory used to store a (file_name, mtime) => digest cache.
    cache_dir: Option<String>,

    /// Prints all the files used to calculate the digest
    #[arg(long)]
    print_files: bool,
}

/// The entry point of "digest-repo" subcommand.
pub fn digest_repo_main(board: &str, root_dir: &Path, args: Args) -> Result<()> {
    // TODO: Implement
    // 1) Find the root overlay in one of the following paths:
    //     * `src/private-overlays/overlay-{board}-private`
    //     * `src/overlays/overlay-{board}
    //    Instead of looking up the private board first, what if we required
    //    board to specify the -private suffix? i.e., grunt-private, grunt-kernelnext-private.
    //    This would make it clear you are working with a private board, it would also make it
    //    easy for developers to test public builds. Currently developers need to check out
    //    a different manifest in order to do this, which means public builds get neglected.
    // 2) Once we have the root overlay, traverse the parents
    // 3) Once we have all the parents walk all the directories collecting all the file names
    //    and their mtimes.
    // 4) Lookup each filename + mtime in the cache, if it's missing compute the hash and store
    //    it in the cache map.
    // 5) Create a root hash from the file names and their hashes.
    // 6) Write the cache map to the cache_dir
    // 7) print the root hash
    // Print a timestamp for now so we area always cache busting.

    let lookup = RepositoryLookup::new(
        root_dir,
        vec!["src/private-overlays", "src/overlays", "src/third_party"],
    )?;

    let repo = lookup.create_repository_set(board)?;

    let digest = repo.digest()?;

    if args.print_files {
        for file in digest.file_hashes {
            println!("{:x} {}", file.1, file.0.display());
        }
    }

    println!("{:x}", digest.repo_hash);

    Ok(())
}
