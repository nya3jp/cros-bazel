// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::read_to_string,
    path::Path,
    sync::{Arc, Mutex},
};

use alchemist::{
    bash::BashValue,
    data::PackageSlotKey,
    dependency::{
        package::{PackageAtomDependency, PackageDependency},
        uri::{UriAtomDependency, UriDependency},
    },
    ebuild::PackageDetails,
    resolver::Resolver,
    translate::{package::translate_package_dependency, uri::translate_uri_dependencies},
};
use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;
use rayon::prelude::*;
use rpds::{HashTrieSetSync, VectorSync};
use serde::{Deserialize, Serialize};
use url::Url;

/// Similar to [PackageData], but post-dependencies are still unresolved.
/// It is used in the middle of computing the dependency graph.
#[derive(Clone, Debug)]
struct PackageDataUnresolved {
    details: Arc<PackageDetails>,
    build_deps: Vec<PackageSlotKey>,
    runtime_deps: Vec<PackageSlotKey>,
    unresolved_post_deps: Vec<PackageAtomDependency>,
    remote_sources: Vec<RemoteSourceData>,
}

/// Represents a package in the dependency graph.
/// It contains [PackageDetails] as well as resolved dependency info.
#[derive(Clone, Debug)]
pub struct PackageData {
    pub details: Arc<PackageDetails>,
    pub build_deps: Vec<PackageSlotKey>,
    pub runtime_deps: Vec<PackageSlotKey>,
    pub post_deps: Vec<PackageSlotKey>,
    pub remote_sources: Vec<RemoteSourceData>,
}

/// Represents a source code archive to be fetched remotely to build a package.
#[derive(Clone, Debug)]
pub struct RemoteSourceData {
    pub urls: Vec<Url>,
    pub filename: String,
    pub size: u64,
    pub hashes: HashMap<String, String>,
}

impl RemoteSourceData {
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

/// Track the path of package dependency atoms being resolved while searching
/// the dependency graph. Its main purpose is to detect circular dependencies.
///
/// This type is immutable and clonable so that it can be used across threads.
#[derive(Clone, Debug)]
struct SearchPath {
    path: VectorSync<PackageAtomDependency>,
    set: HashTrieSetSync<PackageAtomDependency>,
}

impl SearchPath {
    /// Creates an empty [SearchPath].
    fn new() -> Self {
        Self {
            path: VectorSync::new_sync(),
            set: HashTrieSetSync::new_sync(),
        }
    }

    /// Returns an iterator that returns visited package dependency atoms in
    /// the order.
    fn iter(&self) -> impl Iterator<Item = &PackageAtomDependency> {
        self.path.iter()
    }

    /// Returns if the path contains the specified package dependency atom
    /// already.
    fn contains(&self, atom: &PackageAtomDependency) -> bool {
        self.set.contains(atom)
    }

    /// Tries pushing a new package dependency atom to the path and returns a
    /// new [SearchPath] without modifying the original path. It fails if the
    /// same package dependency atom already exists in the path.
    fn try_push(&self, atom: &PackageAtomDependency) -> Result<SearchPath> {
        if self.contains(&atom) {
            bail!(
                "A loop found on searching dependencies:\n\t{}\n\t{}",
                self.iter().map(|atom| atom.to_string()).join(" ->\n\t"),
                atom.to_string()
            );
        }
        Ok(Self {
            path: self.path.push_back(atom.clone()),
            set: self.set.insert(atom.clone()),
        })
    }
}

/// Represents a package dependency type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum DependencyKind {
    /// Build-time dependencies, aka "DEPEND" in Portage.
    Build,
    /// Run-time dependencies, aka "RDEPEND" in Portage.
    Run,
    /// Post-time dependencies, aka "PDEPEND" in Portage.
    Post,
}

fn extract_package_dependencies(
    resolver: &Resolver,
    details: &PackageDetails,
    kind: DependencyKind,
) -> Result<Vec<PackageAtomDependency>> {
    let var_name = match kind {
        DependencyKind::Build => "DEPEND",
        DependencyKind::Run => "RDEPEND",
        DependencyKind::Post => "PDEPEND",
    };

    let raw_deps = match details.vars.get(var_name) {
        None => "",
        Some(BashValue::Scalar(s)) => s.as_str(),
        Some(other) => bail!("Incorrect value for {}: {:?}", var_name, other),
    };

    let deps = raw_deps.parse::<PackageDependency>()?;
    translate_package_dependency(deps, &details.use_map, resolver)
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

fn extract_remote_sources(details: &PackageDetails) -> Result<Vec<RemoteSourceData>> {
    // Collect URIs from SRC_URI.
    let src_uri = match details.vars.get("SRC_URI") {
        None => "",
        Some(BashValue::Scalar(s)) => s.as_str(),
        Some(other) => bail!("Incorrect value for SRC_URI: {:?}", other),
    };
    let source_deps = src_uri.parse::<UriDependency>()?;
    let source_atoms = translate_uri_dependencies(source_deps, &details.use_map)?;

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
            Ok(RemoteSourceData {
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

fn select_package(
    resolver: &Resolver,
    selection: &Mutex<HashMap<PackageSlotKey, PackageDataUnresolved>>,
    path: &SearchPath,
    atom: &PackageAtomDependency,
) -> Result<Option<PackageSlotKey>> {
    let path = path.try_push(atom)?;

    if let Some(_) = resolver.find_provided_packages(&atom).next() {
        return Ok(None);
    }

    let details = resolver.find_best_package(&atom)?;
    let key = details.slot_key();

    // Have we already selected a package for the slot? If so, ensure that there
    // is no inconsistency and return the cached package.
    {
        let selection_guard = selection.lock().unwrap();
        if let Some(old_package) = selection_guard.get(&key) {
            if details.version != old_package.details.version {
                bail!(
                    "Could not resolve non-trivial package selection for {}: {} and {}",
                    &details.package_name,
                    &details.version,
                    &old_package.details.version
                );
            }
            return Ok(Some(key));
        }
    }

    // Extract dependencies.
    let unresolved_build_deps =
        extract_package_dependencies(resolver, &*details, DependencyKind::Build).with_context(
            || {
                format!(
                    "Resolving build-time dependencies for {}-{}",
                    &details.package_name, &details.version
                )
            },
        )?;
    let build_deps = select_packages(resolver, selection, &path, unresolved_build_deps)?;

    let unresolved_runtime_deps =
        extract_package_dependencies(resolver, &*details, DependencyKind::Run).with_context(
            || {
                format!(
                    "Resolving runtime dependencies for {}-{}",
                    &details.package_name, &details.version
                )
            },
        )?;
    let runtime_deps = select_packages(resolver, selection, &path, unresolved_runtime_deps)?;

    let unresolved_post_deps =
        extract_package_dependencies(resolver, &*details, DependencyKind::Post).with_context(
            || {
                format!(
                    "Resolving post-time dependencies for {}-{}",
                    &details.package_name, &details.version
                )
            },
        )?;

    // Extract source URIs.
    let remote_sources = extract_remote_sources(&*details).with_context(|| {
        format!(
            "Resolving remote source dependencies for {}-{}",
            &details.package_name, &details.version
        )
    })?;

    // We may have selected some other packages on resolving dependencies.
    // Check the selection consistency again.
    {
        let mut selection_guard = selection.lock().unwrap();
        if let Some(old_package) = selection_guard.get(&key) {
            if details.version != old_package.details.version {
                bail!(
                    "Could not resolve non-trivial package selection for {}: {} and {}",
                    &details.package_name,
                    &details.version,
                    &old_package.details.version
                );
            }
        } else {
            let new_package = PackageDataUnresolved {
                details,
                build_deps,
                runtime_deps,
                unresolved_post_deps,
                remote_sources,
            };
            eprintln!(
                "Selected: {}-{}:{}",
                &new_package.details.package_name,
                &new_package.details.version,
                &new_package.details.slot
            );
            selection_guard.insert(key.clone(), new_package);
        }
    }

    Ok(Some(key))
}

fn select_packages(
    resolver: &Resolver,
    selection: &Mutex<HashMap<PackageSlotKey, PackageDataUnresolved>>,
    path: &SearchPath,
    atoms: impl IntoParallelIterator<Item = PackageAtomDependency>,
) -> Result<Vec<PackageSlotKey>> {
    Ok(atoms
        .into_par_iter()
        .map(|atom| select_package(resolver, selection, &path.clone(), &atom))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .sorted()
        .dedup()
        .collect_vec())
}

fn resolve_package(
    resolver: &Resolver,
    selection: &Mutex<HashMap<PackageSlotKey, PackageDataUnresolved>>,
    path: &SearchPath,
    unresolved: PackageDataUnresolved,
) -> Result<PackageData> {
    let post_deps = select_packages(resolver, selection, path, unresolved.unresolved_post_deps)?;
    Ok(PackageData {
        details: unresolved.details,
        build_deps: unresolved.build_deps,
        runtime_deps: unresolved.runtime_deps,
        post_deps,
        remote_sources: unresolved.remote_sources,
    })
}

fn analyze_dependency_graph(
    resolver: &Resolver,
    starts: impl IntoIterator<Item = PackageAtomDependency>,
) -> Result<HashMap<PackageSlotKey, PackageData>> {
    let starts = starts.into_iter().collect_vec();

    let path = SearchPath::new();
    let selection: Mutex<HashMap<PackageSlotKey, PackageDataUnresolved>> = Default::default();
    let resolution: Mutex<HashMap<PackageSlotKey, PackageData>> = Default::default();

    select_packages(&resolver, &selection, &path, starts)?;

    loop {
        let to_resolve = {
            let selection_guard = selection.lock().unwrap();
            let selection_keys = selection_guard.keys().collect::<HashSet<_>>();
            let resolution_guard = resolution.lock().unwrap();
            let resolution_keys = resolution_guard.keys().collect::<HashSet<_>>();
            selection_keys
                .difference(&resolution_keys)
                .map(|key| (*key).clone())
                .collect_vec()
        };

        if to_resolve.is_empty() {
            break;
        }

        for key in to_resolve {
            let unresolved = {
                let selection_guard = selection.lock().unwrap();
                (*selection_guard.get(&key).unwrap()).clone()
            };

            let resolved = resolve_package(&resolver, &selection, &path, unresolved)?;

            {
                let mut resolution_guard = resolution.lock().unwrap();
                resolution_guard.insert(key, resolved);
            }
        }
    }

    let resolution = resolution.into_inner().unwrap();

    eprintln!("Selected {} packages", resolution.len());

    Ok(resolution)
}

/// Defines the schema of a package information in the dependency graph JSON.
#[derive(Serialize, Deserialize)]
struct PackageInfo {
    /// Name of the package, e.g. "chromeos-base/chromeos-chrome".
    #[serde(rename = "name")]
    name: String,
    /// Main slot name, e.g. "0".
    #[serde(rename = "mainSlot")]
    main_slot: String,
    /// Path to the ebuild file defining this package, relative from the "src"
    /// directory of CrOS source cehckout, e.g.
    /// "third_party/chromiumos-overlay/chromeos-base/chromeos-chrome/chromeos-chrome-9999.ebuild".
    #[serde(rename = "ebuildPath")]
    ebuild_path: String,
    /// Version of the package, e.g. "9999".
    #[serde(rename = "version")]
    version: String,
    /// Build-time dependencies in the form of Bazel labels, e.g.
    /// "//third_party/chromiumos-overlay/app-accessibility/brltty:0".
    #[serde(rename = "buildDeps")]
    build_deps: Vec<String>,
    /// Run-time dependencies in the form of Bazel labels, e.g.
    /// "//third_party/chromiumos-overlay/app-accessibility/brltty:0".
    #[serde(rename = "runtimeDeps")]
    runtime_deps: Vec<String>,
    /// Distfiles needed to be fetched to build this package.
    #[serde(rename = "srcUris")]
    src_uris: BTreeMap<String, DistFileInfo>,
    /// Post-time dependencies in the form of Bazel labels, e.g.
    /// "//third_party/chromiumos-overlay/app-accessibility/brltty:0".
    #[serde(rename = "postDeps", skip_serializing_if = "Vec::is_empty")]
    post_deps: Vec<String>,
}

/// Defines the schema of a distfile information in the dependency graph JSON.
#[derive(Serialize, Deserialize)]
struct DistFileInfo {
    /// URIs where this distfile can be fetched.
    #[serde(rename = "uris")]
    uris: Vec<String>,
    /// Size of the distfile.
    #[serde(rename = "size")]
    size: u64,
    /// Expected checksum of the distfile in Subresource Integrity format.
    /// https://bazel.build/rules/lib/repo/http?hl=en#http_archive-integrity
    #[serde(rename = "integrity")]
    integrity: String,
    /// SHA256 hash of the distfile. If unavailable, it is set to an empty string.
    // Our version of Bazel doesn't support integrity on http_file, only http_archive
    // so we need to plumb in the hashes.
    /// TODO: Remove this field once we can use integrity everywhere.
    #[serde(rename = "SHA256")]
    sha256: String,
    /// SHA512 hash of the distfile. If unavailable, it is set to an empty string.
    // If we don't have a SHA256 we will use the SHA512 to verify the downloaded file
    // and then compute the SHA256.
    /// TODO: Remove this field once we can use integrity everywhere.
    #[serde(rename = "SHA512")]
    sha512: String,
}

fn compute_label_map(
    graph: &HashMap<PackageSlotKey, PackageData>,
) -> HashMap<PackageSlotKey, String> {
    graph
        .iter()
        .map(|(key, data)| {
            let ebuild_path = data.details.ebuild_path.to_string_lossy();
            let relative_ebuild_path = ebuild_path.split("/src/").last().unwrap();
            let relative_ebuild_components = relative_ebuild_path.split('/').collect_vec();
            let label = format!(
                "//{}:{}",
                relative_ebuild_components[..relative_ebuild_components.len() - 1].join("/"),
                &data.details.slot.main
            );
            (key.clone(), label)
        })
        .collect()
}

/// The entry point of "dump-deps" subcommand.
pub fn dump_deps_main(resolver: &Resolver, starts: Vec<PackageAtomDependency>) -> Result<()> {
    let graph = analyze_dependency_graph(resolver, starts)?;

    let label_map = compute_label_map(&graph);

    let data: BTreeMap<String, PackageInfo> = graph
        .iter()
        .map(|(key, data)| {
            let info = PackageInfo {
                name: data.details.package_name.clone(),
                main_slot: data.details.slot.main.clone(),
                ebuild_path: data
                    .details
                    .ebuild_path
                    .strip_prefix("/mnt/host/source/src")
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                version: data.details.version.to_string(),
                build_deps: data
                    .build_deps
                    .iter()
                    .map(|key| label_map.get(key).unwrap().clone())
                    .sorted()
                    .collect(),
                runtime_deps: data
                    .runtime_deps
                    .iter()
                    .map(|key| label_map.get(key).unwrap().clone())
                    .sorted()
                    .collect(),
                post_deps: data
                    .post_deps
                    .iter()
                    .map(|key| label_map.get(key).unwrap().clone())
                    .sorted()
                    .collect(),
                src_uris: data
                    .remote_sources
                    .iter()
                    .map(|source| {
                        let integrity = source.compute_integrity()?;
                        let info = DistFileInfo {
                            uris: source.urls.iter().map(|uri| uri.to_string()).collect(),
                            size: source.size,
                            integrity,
                            sha512: source
                                .hashes
                                .get("SHA512")
                                .map(|s| s.to_owned())
                                .unwrap_or_default(),
                            sha256: source
                                .hashes
                                .get("SHA256")
                                .map(|s| s.to_owned())
                                .unwrap_or_default(),
                        };
                        Ok((source.filename.to_owned(), info))
                    })
                    .collect::<Result<_>>()?,
            };
            let label = label_map.get(key).unwrap().to_owned();
            Ok((label, info))
        })
        .collect::<Result<_>>()?;

    serde_json::to_writer_pretty(std::io::stdout(), &data)?;
    Ok(())
}
