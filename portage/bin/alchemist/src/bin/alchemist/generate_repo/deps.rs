// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{fs::File, path::Path};

use alchemist::analyze::{source::PackageLocalSource, Package};
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
        gclient: String,
    },
}

pub fn generate_deps_file(packages: &[&Package], out: &Path) -> Result<()> {
    let repos = generate_deps(packages)?;
    let mut file = File::create(out)?;
    serde_json::to_writer(&mut file, &repos)?;
    Ok(())
}

#[instrument(skip_all)]
fn generate_deps(packages: &[&Package]) -> Result<Vec<Repository>> {
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
            gclient: "@depot_tools//:gclient.wrapper.sh".to_string(),
        });

    Ok(unique_dists.chain(repos).chain(chrome).collect())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        path::PathBuf,
        sync::Arc,
    };

    use alchemist::{
        analyze::{
            dependency::PackageDependencies,
            source::{PackageDistSource, PackageSources},
        },
        bash::vars::BashVars,
        data::{Slot, UseMap},
        ebuild::{
            metadata::{EBuildBasicData, EBuildMetadata},
            PackageDetails, PackageReadiness,
        },
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

        let dependencies = PackageDependencies {
            build_deps: vec![],
            test_deps: vec![],
            runtime_deps: vec![],
            post_deps: vec![],
            build_host_deps: vec![],
            install_host_deps: vec![],
        };

        let make_details = |short_package_name: &str| PackageDetails {
            metadata: Arc::new(EBuildMetadata {
                basic_data: EBuildBasicData {
                    repo_name: "baz".to_owned(),
                    ebuild_path: PathBuf::from(format!(
                        "/somewhere/sys-apps/{short_package_name}-1.0.ebuild"
                    )),
                    package_name: format!("sys-apps/{short_package_name}"),
                    short_package_name: short_package_name.to_owned(),
                    category_name: "sys-apps".to_owned(),
                    version: Version::try_new("1.0").unwrap(),
                },
                vars: BashVars::new(HashMap::new()),
            }),
            slot: Slot::new("0"),
            use_map: UseMap::new(),
            stable: true,
            readiness: PackageReadiness::Ok,
            inherited: HashSet::new(),
            inherit_paths: vec![],
            direct_build_target: None,
        };

        let details1 = make_details("p1");
        let details2 = make_details("p2");
        let details3 = make_details("p3");

        let packages = vec![
            Package {
                details: details1.into(),
                dependencies: dependencies.clone(),
                sources: cipd_sources,
                install_set: vec![],
                build_host_deps: vec![],
                bashrcs: vec![],
            },
            Package {
                details: details2.into(),
                dependencies: dependencies.clone(),
                sources: gs_sources,
                install_set: vec![],
                build_host_deps: vec![],
                bashrcs: vec![],
            },
            Package {
                details: details3.into(),
                dependencies: dependencies.clone(),
                sources: https_sources,
                install_set: vec![],
                build_host_deps: vec![],
                bashrcs: vec![],
            },
        ];

        let repos = generate_deps(&packages.iter().collect_vec())?;
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
