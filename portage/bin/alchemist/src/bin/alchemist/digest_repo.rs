// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::alchemist::TargetData;
use alchemist::repository::{RepositoryDigest, UnorderedRepositorySet};
use anyhow::Result;

#[derive(clap::Args, Clone, Debug)]
pub struct Args {
    /// Directory used to store a (file_name, mtime) => digest cache.
    cache_dir: Option<String>,

    /// Prints all the files used to calculate the digest
    #[arg(long)]
    print_files: bool,
}

/// The entry point of "digest-repo" subcommand.
pub fn digest_repo_main(host: &TargetData, target: Option<&TargetData>, args: Args) -> Result<()> {
    let repos: UnorderedRepositorySet = [
        host.repos.get_repos(),
        target.map_or(vec![], |data| data.repos.get_repos()),
    ]
    .into_iter()
    .flat_map(|x| x.into_iter())
    .cloned()
    .collect();

    let sources = [
        host.config.sources(),
        target.map_or(vec![], |data| data.config.sources()),
    ]
    .into_iter()
    .flatten()
    .collect();

    let digest = RepositoryDigest::new(&repos, sources)?;

    if args.print_files {
        for file in digest.file_hashes {
            println!("{:x} {}", file.1, file.0.display());
        }
    }

    println!("{:x}", digest.repo_hash);

    Ok(())
}
