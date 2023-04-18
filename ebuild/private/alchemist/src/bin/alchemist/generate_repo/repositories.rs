// Copyright 2023 The ChromiumOS Authors
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

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use alchemist::{
        analyze::{
            dependency::PackageDependencies,
            source::{PackageDistSource, PackageSources},
        },
        bash::vars::BashVars,
        data::Slot,
        ebuild::{PackageDetails, Stability},
    };
    use url::Url;
    use version::Version;

    use super::*;

    // TODO(b/273158038): Convert to a golden test. See example in
    // ebuild/private/alchemist/src/bin/alchemist/generate_repo/internal/sources/mod.rs
    #[test]
    fn generate_repositories_file_succeeds() -> Result<()> {
        let hashes = HashMap::from([("SHA256".to_string(), "012346".to_string())]);

        let cipd_sources = PackageSources {
            local_sources: vec![],
            repo_sources: vec![],
            dist_sources: vec![PackageDistSource {
                urls: vec![Url::parse("cipd://skia/tools/goldctl/linux-amd64:0ov3TU").unwrap()],
                filename: "goldctl-2021.03.31-amd64.zip".to_owned(),
                size: 100,
                hashes: hashes.clone(),
            }],
        };

        let https_sources = PackageSources {
            local_sources: vec![],
            repo_sources: vec![],
            dist_sources: vec![
                PackageDistSource {
                    urls: vec![
                        Url::parse("https://commondatastorage.googleapis.com/chromeos-localmirror/distfiles/google-api-core-1.19.0.tar.gz").unwrap()
                    ],
                    filename: "google-api-core-1.19.0.tar.gz".to_owned(),
                    size: 100,
                    hashes: hashes.clone(),
                },
            ],
        };

        let dependencies = PackageDependencies {
            build_deps: vec![],
            runtime_deps: vec![],
            post_deps: vec![],
        };

        let details_prototype = PackageDetails {
            package_name: "prototype".to_owned(),
            version: Version::try_new("1.0").unwrap(),
            vars: BashVars::new(HashMap::new()),
            slot: Slot::new("0"),
            use_map: HashMap::new(),
            stability: Stability::Stable,
            masked: false,
            ebuild_path: "/somewhere/sys-apps/prototype-1.0.ebuild".into(),
            inherited: HashSet::new(),
        };

        let mut details_one = details_prototype.clone();
        details_one.package_name = "sys-apps/one".to_owned();
        details_one.ebuild_path = "/somewhere/sys-apps/one-1.0.ebuild".into();

        let mut details_two = details_prototype.clone();
        details_two.package_name = "sys-apps/two".to_owned();
        details_two.ebuild_path = "/somewhere/sys-apps/two-1.0.ebuild".into();

        let packages = vec![
            Package {
                details: details_one.into(),
                dependencies: dependencies.clone(),
                sources: cipd_sources,
                install_set: vec![],
            },
            Package {
                details: details_two.into(),
                dependencies: dependencies.clone(),
                sources: https_sources,
                install_set: vec![],
            },
        ];

        let output_file = tempfile::NamedTempFile::new()?;
        generate_repositories_file(&packages, output_file.path())?;

        let actual_output = std::fs::read_to_string(output_file.path())?;
        // Final `(` makes sure that we don't match `load` calls.
        assert!(actual_output.contains("cipd_file("));
        assert!(actual_output.contains("http_file("));

        Ok(())
    }
}
