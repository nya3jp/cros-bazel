// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{fs::File, io::Write, path::Path};

use anyhow::Result;
use itertools::Itertools;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use super::common::{DistFileEntry, Package, AUTOGENERATE_NOTICE};

static REPOSITORIES_TEMPLATE: &str = include_str!("repositories-template.bzl");

#[derive(Serialize)]
struct RepositoriesTemplateContext {
    dists: Vec<DistFileEntry>,
}

pub fn generate_repositories_file(packages: &Vec<Package>, out: &Path) -> Result<()> {
    let joined_dists: Vec<DistFileEntry> = packages
        .iter()
        .flat_map(|package| {
            package
                .sources
                .remote_sources
                .iter()
                .map(DistFileEntry::try_new)
        })
        .collect::<Result<_>>()?;

    let unique_dists = joined_dists
        .into_iter()
        .sorted_by(|a, b| a.filename.cmp(&b.filename))
        .dedup_by(|a, b| a.filename == b.filename)
        .collect();

    let context = RepositoriesTemplateContext {
        dists: unique_dists,
    };

    let mut templates = TinyTemplate::new();
    templates.add_template("main", REPOSITORIES_TEMPLATE)?;
    let rendered = templates.render("main", &context)?;

    let mut file = File::create(out)?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    file.write_all(rendered.as_bytes())?;
    Ok(())
}
