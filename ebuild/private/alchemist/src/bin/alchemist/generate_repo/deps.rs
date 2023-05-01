// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{fs::File, path::Path};

use alchemist::analyze::source::PackageLocalSource;
use anyhow::Result;
use itertools::Itertools;
use serde::Serialize;
use tracing::instrument;

use super::common::{DistFileEntry, Package};

// Each entry here corresponds to a repository rule, and the fields in the
// struct must correspond to the parameters to that repository rule.
#[derive(Serialize, Debug)]
enum Repository {
    CipdFile {
        name: String,
        downloaded_file_path: String,
        url: String,
    },
    HttpFile {
        name: String,
        downloaded_file_path: String,
        integrity: String,
        urls: Vec<String>,
    },
    RepoRepository {
        name: String,
        project: String,
        tree: String,
    },
    CrosChromeRepository {
        name: String,
        tag: String,
        gclient: String,
    },
}

pub fn generate_deps_file(packages: &[Package], out: &Path) -> Result<()> {
    let repos = generate_deps(packages)?;
    let mut file = File::create(out)?;
    serde_json::to_writer(&mut file, &repos)?;
    Ok(())
}

#[instrument(skip_all)]
fn generate_deps(packages: &[Package]) -> Result<Vec<Repository>> {
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
        .map(|dist| {
            let url = &dist.urls[0];
            if url.starts_with("cipd") {
                Repository::CipdFile {
                    name: dist.repository_name,
                    downloaded_file_path: dist.filename,
                    url: url.to_string(),
                }
            } else {
                Repository::HttpFile {
                    name: dist.repository_name,
                    downloaded_file_path: dist.filename,
                    integrity: dist.integrity,
                    urls: dist.urls.clone(),
                }
            }
        });

    let repos = packages
        .iter()
        .flat_map(|package| &package.sources.repo_sources)
        .unique_by(|source| &source.name)
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .map(|repo| Repository::RepoRepository {
            name: repo.name.clone(),
            project: repo.project.clone(),
            tree: repo.tree_hash.clone(),
        });

    let chrome = packages
        .iter()
        .flat_map(|package| &package.sources.local_sources)
        .filter_map(|origin| match origin {
            PackageLocalSource::Chrome(version) => Some(version),
            _ => None,
        })
        .unique()
        .sorted()
        .map(|version| Repository::CrosChromeRepository {
            name: format!("chrome-{}", version),
            tag: version.clone(),
            gclient: "@depot_tools//:gclient_wrapper.sh".to_string(),
        });

    Ok(unique_dists.chain(repos).chain(chrome).collect())
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
        ebuild::PackageDetails,
    };
    use pretty_assertions::assert_eq;
    use url::Url;
    use version::Version;

    use super::*;

    #[test]
    fn generate_deps_succeeds() -> Result<()> {
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
            repo_name: "baz".to_owned(),
            package_name: "prototype".to_owned(),
            version: Version::try_new("1.0").unwrap(),
            vars: BashVars::new(HashMap::new()),
            slot: Slot::new("0"),
            use_map: HashMap::new(),
            accepted: true,
            stable: true,
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

        let repos = generate_deps(&packages)?;
        let actual = serde_json::to_string_pretty(&repos)?;
        let expected = r#"[
  {
    "CipdFile": {
      "name": "portage-dist_goldctl-2021.03.31-amd64.zip",
      "downloaded_file_path": "goldctl-2021.03.31-amd64.zip",
      "url": "cipd://skia/tools/goldctl/linux-amd64:0ov3TU"
    }
  },
  {
    "HttpFile": {
      "name": "portage-dist_google-api-core-1.19.0.tar.gz",
      "downloaded_file_path": "google-api-core-1.19.0.tar.gz",
      "integrity": "sha256-ASNG",
      "urls": [
        "https://commondatastorage.googleapis.com/chromeos-localmirror/distfiles/google-api-core-1.19.0.tar.gz"
      ]
    }
  }
]"#;
        assert_eq!(actual, expected);

        Ok(())
    }
}
