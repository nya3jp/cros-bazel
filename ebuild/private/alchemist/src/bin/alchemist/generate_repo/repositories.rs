// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{fs::File, io::Write, path::Path};

use alchemist::analyze::source::PackageLocalSource;
use anyhow::Result;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use tera::{Context, Tera};

use super::common::{DistFileEntry, Package, AUTOGENERATE_NOTICE};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template(
            "repositories.bzl",
            include_str!("templates/repositories.bzl"),
        )
        .unwrap();
        tera
    };
}

#[derive(Serialize)]
struct RepoRepositoryTemplateContext {
    name: String,
    project: String,
    tree_hash: String,
}

#[derive(Serialize)]
struct RepositoriesTemplateContext<'a> {
    dists: Vec<DistFileEntry>,
    repos: Vec<RepoRepositoryTemplateContext>,
    chrome: Vec<&'a String>,
}

pub fn generate_repositories_file(packages: &[Package], out: &Path) -> Result<()> {
    let joined_dists: Vec<DistFileEntry> = packages
        .iter()
        .flat_map(|package| {
            package
                .sources
                .dist_sources
                .iter()
                .map(DistFileEntry::try_new)
        })
        .collect::<Result<_>>()?;

    let unique_dists = joined_dists
        .into_iter()
        .sorted_by(|a, b| a.filename.cmp(&b.filename))
        .dedup_by(|a, b| a.filename == b.filename)
        .collect();

    let repos: Vec<RepoRepositoryTemplateContext> = packages
        .iter()
        .flat_map(|package| &package.sources.repo_sources)
        .unique_by(|source| &source.name)
        .map(|repo| RepoRepositoryTemplateContext {
            name: repo.name.clone(),
            project: repo.project.clone(),
            tree_hash: repo.tree_hash.clone(),
        })
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect();

    let chrome = packages
        .iter()
        .flat_map(|package| &package.sources.local_sources)
        .filter_map(|origin| match origin {
            PackageLocalSource::Chrome(version) => Some(version),
            _ => None,
        })
        .unique()
        .sorted()
        .collect();

    let context = RepositoriesTemplateContext {
        dists: unique_dists,
        repos,
        chrome,
    };

    let mut file = File::create(out)?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to("repositories.bzl", &Context::from_serialize(context)?, file)?;

    Ok(())
}
