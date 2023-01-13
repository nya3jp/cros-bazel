// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    fs::read_to_string,
    iter::{repeat, zip},
    path::Path,
};

use anyhow::{anyhow, bail, Result};
use itertools::Itertools;
use url::Url;

use crate::{
    bash::BashValue,
    data::UseMap,
    dependency::{
        algorithm::{elide_use_conditions, parse_simplified_dependency, simplify},
        uri::{UriAtomDependency, UriDependency},
    },
    ebuild::PackageDetails,
};

/// Represents an origin of local source code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PackageLocalSourceOrigin {
    /// ChromeOS source code at `/mnt/host/source/src`.
    Src,
    /// Chromite source code at `/mnt/host/source/chromite`.
    Chromite,
    /// Chrome source code.
    Chrome,
}

/// Represents local source code.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageLocalSource {
    /// Origin of this source code.
    pub origin: PackageLocalSourceOrigin,
    /// The directory containg source code, relative to the root of the origin.
    /// Empty string means all source code in the origin. The path must not end
    /// with a slash.
    pub path: String,
}

impl PackageLocalSource {
    pub fn starts_with(&self, prefix: &Self) -> bool {
        if self.origin != prefix.origin {
            return false;
        }
        self.path == prefix.path || self.path.starts_with(&format!("{}/", &prefix.path))
    }
}

/// Represents a source code archive to be fetched remotely to build a package.
#[derive(Clone, Debug)]
pub struct PackageRemoteSource {
    pub urls: Vec<Url>,
    pub filename: String,
    pub size: u64,
    pub hashes: HashMap<String, String>,
}

impl PackageRemoteSource {
    pub fn compute_integrity(&self) -> Result<String> {
        // We prefer SHA512 for integrity checking.
        for name in ["SHA512", "SHA256", "BLAKE2B"] {
            let hash_hex = match self.hashes.get(name) {
                Some(hash) => hash,
                None => continue,
            };
            let hash_bytes = hex::decode(hash_hex)?;
            let hash_base64 = base64::encode(hash_bytes);
            return Ok(format!("{}-{}", name.to_ascii_lowercase(), hash_base64));
        }
        bail!("No supported hash found in Manifest");
    }
}

/// Analyzed source information of a package. It is returned by
/// [`analyze_sources`].
pub struct PackageSources {
    pub local_sources: Vec<PackageLocalSource>,
    pub remote_sources: Vec<PackageRemoteSource>,
}

fn get_cros_workon_array_variable(
    details: &PackageDetails,
    name: &str,
    projects: usize,
) -> Result<Vec<String>> {
    let raw_values = match details.vars.hash_map().get(name) {
        None => {
            bail!("{} not defined", name);
        }
        Some(BashValue::Scalar(value)) => vec![value.clone()],
        Some(BashValue::IndexedArray(values)) => values.clone(),
        Some(other) => {
            bail!("Invalid {} value: {:?}", name, other);
        }
    };

    // If the number of elements is 1, repeat the same value for the number of
    // projects.
    let extended_values = if raw_values.len() == 1 {
        repeat(raw_values[0].clone()).take(projects).collect()
    } else {
        raw_values
    };
    Ok(extended_values)
}

fn extract_cros_workon_sources(details: &PackageDetails) -> Result<Vec<PackageLocalSource>> {
    let projects = match details.vars.hash_map().get("CROS_WORKON_PROJECT") {
        None => {
            // This is not a cros-workon package.
            return Ok(Vec::new());
        }
        Some(BashValue::Scalar(project)) => vec![project.clone()],
        Some(BashValue::IndexedArray(projects)) => projects.clone(),
        others => {
            bail!("Invalid CROS_WORKON_PROJECT value: {:?}", others)
        }
    };

    let local_names =
        get_cros_workon_array_variable(details, "CROS_WORKON_LOCALNAME", projects.len())?;
    let subtrees = get_cros_workon_array_variable(details, "CROS_WORKON_SUBTREE", projects.len())?;

    let is_chromeos_base = details.package_name.starts_with("chromeos-base/");

    let mut source_paths = Vec::<String>::new();

    for (local_name, subtree) in zip(local_names, subtrees) {
        // CROS_WORKON_LOCALNAME points to file paths relative to src/ if the
        // package is in the chromeos-base category; otherwise they're relative
        // to src/third_party/.
        let local_path = if local_name == "chromiumos-assets" {
            // HACK: chromiumos-assets ebuild is incorrect.
            // TODO: Fix the ebuild and remove this hack.
            "platform/chromiumos-assets".to_owned()
        } else if is_chromeos_base {
            local_name
        } else if let Some(clean_path) = local_name.strip_prefix("../") {
            clean_path.to_owned()
        } else {
            format!("third_party/{}", local_name)
        };

        // Consider CROS_WORKON_SUBTREE for platform2 packages only.
        if subtree.is_empty() || !local_path.starts_with("platform2") {
            // HACK: We need a pinned version of crosvm for sys_util_core, so we
            // can't use the default location.
            // TODO: Inspect CROS_WORKON_MANUAL_UPREV to detect pinned packages
            // automatically and remove this hack.
            if details.package_name == "dev-rust/sys_util_core" && local_path == "platform/crosvm" {
                source_paths.push("platform/crosvm-sys_util_core".to_owned());
            } else {
                source_paths.push(local_path);
            }
        } else {
            let subtree_local_paths = subtree.split_ascii_whitespace().flat_map(|subtree_path| {
                // TODO: Remove these special cases.
                match subtree_path {
                    // Use the platform2 src package instead.
                    ".gn" => Some(local_path.clone()),
                    // We really don't need .clang-format to build...
                    ".clang-format" => None,
                    // We don't have a sub-package for platform2/chromeos-config.
                    "chromeos-config/cros_config_host" => {
                        Some(format!("{}/chromeos-config", &local_path))
                    }
                    _ => Some(format!("{}/{}", &local_path, &subtree_path)),
                }
            });
            source_paths.extend(subtree_local_paths);
        }
    }

    let mut sources = source_paths
        .into_iter()
        .map(|path| PackageLocalSource {
            origin: PackageLocalSourceOrigin::Src,
            path,
        })
        .collect_vec();

    // Kernel packages need extra eclasses.
    // TODO: Remove this hack.
    if projects
        .iter()
        .any(|p| p == "cros/third_party/kernel")
    {
        sources.push(PackageLocalSource {
            origin: PackageLocalSourceOrigin::Src,
            path: "third_party/chromiumos-overlay/eclass/cros-kernel".to_owned(),
        });
    }

    Ok(sources)
}

fn extract_local_sources(details: &PackageDetails) -> Result<Vec<PackageLocalSource>> {
    let mut sources = extract_cros_workon_sources(details)?;

    // Chromium packages need its source code.
    // TODO: Remove this hack.
    if details.inherited.contains("chromium-source") {
        // TODO: We need USE flags to add src-internal.
        sources.push(PackageLocalSource {
            origin: PackageLocalSourceOrigin::Chrome,
            path: String::new(),
        });
    }

    // The platform eclass calls `platform2.py` which requires chromite.
    // The dlc eclass calls `build_dlc` which lives in chromite.
    // dev-libs/gobject-introspection calls `platform2_test.py` which lives in
    // chromite.
    // TODO: Remove this hack.
    if details.inherited.contains("platform")
        || details.inherited.contains("dlc")
        || details.package_name == "dev-libs/gobject-introspection"
    {
        sources.push(PackageLocalSource {
            origin: PackageLocalSourceOrigin::Chromite,
            path: String::new(),
        })
    }

    sources.sort();
    sources.dedup();

    Ok(sources)
}

fn parse_uri_dependencies(deps: UriDependency, use_map: &UseMap) -> Result<Vec<UriAtomDependency>> {
    let deps = elide_use_conditions(deps, &use_map).unwrap_or_default();
    let deps = simplify(deps);
    parse_simplified_dependency(deps)
}

struct DistEntry {
    pub filename: String,
    pub size: u64,
    pub hashes: HashMap<String, String>,
}

struct PackageManifest {
    pub dists: Vec<DistEntry>,
}

fn load_package_manifest(dir: &Path) -> Result<PackageManifest> {
    let content = read_to_string(&dir.join("Manifest"))?;
    let dists = content
        .split('\n')
        .map(|line| {
            let mut columns = line.split_ascii_whitespace().map(|s| s.to_owned());

            let dist = columns.next();
            match dist {
                Some(s) if s == "DIST" => {}
                _ => return Ok(None), // ignore other lines
            }

            let (filename, size_str) = columns
                .next_tuple()
                .ok_or_else(|| anyhow!("Corrupted Manifest line: {}", line))?;
            let size = size_str.parse()?;
            let hashes = HashMap::<String, String>::from_iter(columns.tuples());
            Ok(Some(DistEntry {
                filename,
                size,
                hashes,
            }))
        })
        .flatten_ok()
        .collect::<Result<Vec<_>>>()?;
    Ok(PackageManifest { dists })
}

fn extract_remote_sources(details: &PackageDetails) -> Result<Vec<PackageRemoteSource>> {
    // Collect URIs from SRC_URI.
    let src_uri = details.vars.get_scalar_or_default("SRC_URI")?;
    let source_deps = src_uri.parse::<UriDependency>()?;
    let source_atoms = parse_uri_dependencies(source_deps, &details.use_map)?;

    // Construct a map from file names to URIs.
    let mut source_map = HashMap::<String, Vec<Url>>::new();
    for source_atom in source_atoms {
        let (url, filename) = match source_atom {
            UriAtomDependency::Uri(url, opt_filename) => {
                let filename = if let Some(filename) = opt_filename {
                    filename
                } else if let Some(segments) = url.path_segments() {
                    segments.last().unwrap().to_owned()
                } else {
                    bail!("Invalid source URI: {}", &url);
                };
                (url, filename)
            }
            UriAtomDependency::Filename(filename) => {
                bail!("Found non-URI source: {}", &filename);
            }
        };
        source_map.entry(filename).or_default().push(url);
    }

    if source_map.is_empty() {
        return Ok(Vec::new());
    }

    let manifest = load_package_manifest(&details.ebuild_path.parent().unwrap())?;

    let mut dist_map: HashMap<String, DistEntry> = manifest
        .dists
        .into_iter()
        .map(|dist| (dist.filename.clone(), dist))
        .collect();

    let mut sources = source_map
        .into_iter()
        .map(|(filename, urls)| {
            let dist = dist_map
                .remove(&filename)
                .ok_or_else(|| anyhow!("{} not found in Manifest", &filename))?;
            Ok(PackageRemoteSource {
                urls,
                filename,
                size: dist.size,
                hashes: dist.hashes,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    sources.sort_unstable_by(|a, b| a.filename.cmp(&b.filename));

    Ok(sources)
}

/// Analyzes ebuild variables and returns [`PackageSources`] sumarizing its
/// source information.
pub fn analyze_sources(details: &PackageDetails) -> Result<PackageSources> {
    Ok(PackageSources {
        local_sources: extract_local_sources(details)?,
        remote_sources: extract_remote_sources(details)?,
    })
}

pub fn fixup_sources<'a, I: IntoIterator<Item = &'a mut PackageSources>>(sources_iter: I) {
    // Not all packages use the same level of SUBTREE, some have deeper targets
    // than others. This results in the packages that have a shallower SUBTREE
    // missing out on the files defined in the deeper tree.
    // To fix this we need to populate targets with the shallow tree with
    // all the additional deeper paths.
    //
    // e.g.,
    // iioservice requires //platform2/iioservice:src
    // cros-camera-libs requires //platform2/iioservice/mojo:src
    //
    // When trying to build iioservice the mojo directory will be missing.
    // So this code will add //platform2/iioservice/mojo:src to iioservice.

    let all_sources = sources_iter.into_iter().collect_vec();

    let all_local_sources: Vec<PackageLocalSource> = all_sources
        .iter()
        .flat_map(|source| &source.local_sources)
        .sorted()
        .dedup()
        .map(|source| source.clone())
        .collect();

    let child_sources_map: HashMap<&PackageLocalSource, Vec<&PackageLocalSource>> = {
        let mut child_sources_map: HashMap<&PackageLocalSource, Vec<&PackageLocalSource>> =
            all_local_sources
                .iter()
                .map(|source| (source, Vec::new()))
                .collect();
        let mut parent_sources_stack = Vec::<&PackageLocalSource>::new();

        for current_source in all_local_sources.iter() {
            while let Some(last_parent_source) = parent_sources_stack.pop() {
                if current_source.starts_with(&last_parent_source) {
                    parent_sources_stack.push(last_parent_source);
                    break;
                }
            }

            for parent_source in parent_sources_stack.iter() {
                child_sources_map
                    .get_mut(*parent_source)
                    .unwrap()
                    .push(current_source);
            }

            if !(current_source.origin == PackageLocalSourceOrigin::Src
                && current_source.path == "platform2")
            {
                parent_sources_stack.push(current_source);
            }
        }

        child_sources_map
    };

    for source in all_sources {
        let local_sources = std::mem::take(&mut source.local_sources);
        let local_sources = local_sources
            .into_iter()
            .flat_map(|old_source| {
                let mut new_sources = vec![old_source.clone()];
                if let Some(child_sources) = child_sources_map.get(&old_source) {
                    new_sources.extend(child_sources.iter().map(|source| (*source).clone()));
                }
                new_sources
            })
            .sorted()
            .dedup()
            .collect();
        source.local_sources = local_sources;
    }
}
