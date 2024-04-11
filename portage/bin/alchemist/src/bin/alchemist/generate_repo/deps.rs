// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{fs::File, path::Path};

use alchemist::analyze::source::{ChromeType, PackageLocalSource, PackageSources};
use anyhow::Result;
use itertools::Itertools;
use serde::Serialize;
use tracing::instrument;

use super::common::DistFileEntry;

// Each entry here corresponds to a repository rule, and the fields in the
// struct must correspond to the parameters to that repository rule.
#[derive(Serialize, Debug)]
enum Repository {
    CipdFile {
        name: String,
        downloaded_file_path: String,
        url: String,
    },
    GsFile {
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
    #[allow(clippy::enum_variant_names)]
    RepoRepository {
        name: String,
        project: String,
        tree: String,
    },
    #[allow(clippy::enum_variant_names)]
    CrosChromeRepository {
        name: String,
        tag: String,
        internal: bool,
    },
}

pub fn generate_deps_file(all_sources: &[&PackageSources], out: &Path) -> Result<()> {
    let repos = generate_deps(all_sources)?;
    let mut file = File::create(out)?;
    serde_json::to_writer(&mut file, &repos)?;
    Ok(())
}

#[instrument(skip_all)]
fn generate_deps(all_sources: &[&PackageSources]) -> Result<Vec<Repository>> {
    let joined_dists: Vec<DistFileEntry> = all_sources
        .iter()
        .flat_map(|sources| sources.dist_sources.iter().map(DistFileEntry::try_new))
        .collect::<Result<_>>()?;

    let unique_dists = joined_dists
        .into_iter()
        .sorted_by(|a, b| a.filename.cmp(&b.filename))
        .dedup_by(|a, b| a.filename == b.filename)
        .map(|dist| {
            let url = &dist.urls[0];
            if url.starts_with("cipd://") {
                Repository::CipdFile {
                    name: dist.name,
                    downloaded_file_path: dist.filename,
                    url: url.to_string(),
                }
            } else if url.starts_with("gs://") {
                Repository::GsFile {
                    name: dist.name,
                    downloaded_file_path: dist.filename,
                    url: url.to_string(),
                }
            } else {
                Repository::HttpFile {
                    name: dist.name,
                    downloaded_file_path: dist.filename,
                    integrity: dist.integrity,
                    urls: dist.urls.clone(),
                }
            }
        });

    let repos = all_sources
        .iter()
        .flat_map(|sources| &sources.repo_sources)
        .unique_by(|source| &source.name)
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .map(|repo| Repository::RepoRepository {
            name: repo.name.clone(),
            project: repo.project.clone(),
            tree: repo.tree_hash.clone(),
        });

    let chrome = all_sources
        .iter()
        .flat_map(|sources| &sources.local_sources)
        .filter_map(|origin| match origin {
            PackageLocalSource::Chrome(version, chrome_type) => Some((version, chrome_type)),
            _ => None,
        })
        .unique()
        .sorted()
        .map(|(version, chrome_type)| match chrome_type {
            ChromeType::Public => Repository::CrosChromeRepository {
                name: format!("chrome-{}", version),
                tag: version.clone(),
                internal: false,
            },
            ChromeType::Internal => Repository::CrosChromeRepository {
                name: format!("chrome-internal-{}", version),
                tag: version.clone(),
                internal: true,
            },
        });

    Ok(unique_dists.chain(repos).chain(chrome).collect())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use alchemist::analyze::source::{PackageDistSource, PackageSources};
    use pretty_assertions::assert_eq;
    use url::Url;

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

        let gs_sources = PackageSources {
            local_sources: vec![],
            repo_sources: vec![],
            dist_sources: vec![PackageDistSource {
                urls: vec![Url::parse("gs://secret-bucket/secret-file.tar.gz").unwrap()],
                filename: "secret-file.tar.gz".to_owned(),
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

        let all_sources = vec![&cipd_sources, &gs_sources, &https_sources];

        let repos = generate_deps(&all_sources)?;
        let actual = serde_json::to_string_pretty(&repos)?;
        let expected = r#"[
  {
    "CipdFile": {
      "name": "dist_goldctl-2021.03.31-amd64.zip",
      "downloaded_file_path": "goldctl-2021.03.31-amd64.zip",
      "url": "cipd://skia/tools/goldctl/linux-amd64:0ov3TU"
    }
  },
  {
    "HttpFile": {
      "name": "dist_google-api-core-1.19.0.tar.gz",
      "downloaded_file_path": "google-api-core-1.19.0.tar.gz",
      "integrity": "sha256-ASNG",
      "urls": [
        "https://commondatastorage.googleapis.com/chromeos-localmirror/distfiles/google-api-core-1.19.0.tar.gz"
      ]
    }
  },
  {
    "GsFile": {
      "name": "dist_secret-file.tar.gz",
      "downloaded_file_path": "secret-file.tar.gz",
      "url": "gs://secret-bucket/secret-file.tar.gz"
    }
  }
]"#;
        assert_eq!(actual, expected);

        Ok(())
    }
}
