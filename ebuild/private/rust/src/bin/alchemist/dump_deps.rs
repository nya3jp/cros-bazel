// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::read_to_string,
    iter::{repeat, zip},
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
    pub local_sources: Vec<String>,
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
    pub local_sources: Vec<String>,
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

fn get_extra_dependencies(details: &PackageDetails, kind: DependencyKind) -> &'static str {
    match (details.package_name.as_str(), kind) {
        // poppler seems to support building without Boost, but the build fails
        // without it.
        ("app-text/poppler", DependencyKind::Build) => "dev-libs/boost",
        // m2crypt fails to build for missing Python.h.
        ("dev-python/m2crypto", DependencyKind::Build) => "dev-lang/python:3.6",
        // xau.pc contains "Requires: xproto", so it should be listed as RDEPEND.
        ("x11-libs/libXau", DependencyKind::Run) => "x11-base/xorg-proto",
        _ => "",
    }
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

    let raw_extra_deps = get_extra_dependencies(details, kind);

    let joined_raw_deps = format!("{} {}", raw_deps, raw_extra_deps);
    let deps = joined_raw_deps.parse::<PackageDependency>()?;

    translate_package_dependency(deps, &details.use_map, resolver)
}

fn get_cros_workon_array_variable(
    details: &PackageDetails,
    name: &str,
    projects: usize,
) -> Result<Vec<String>> {
    let raw_values = match details.vars.get(name) {
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

fn extract_cros_workon_sources(details: &PackageDetails) -> Result<Vec<String>> {
    let projects = match details.vars.get("CROS_WORKON_PROJECT") {
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

    let mut source_labels = source_paths
        .into_iter()
        .map(|path| format!("//{}:src", path))
        .collect_vec();

    // Kernel packages need extra eclasses.
    // TODO: Remove this hack.
    if projects
        .iter()
        .any(|p| p == "chromiumos/third_party/kernel")
    {
        source_labels.push("//third_party/chromiumos-overlay/eclass/cros-kernel:src".to_owned());
    }

    Ok(source_labels)
}

fn extract_local_sources(details: &PackageDetails) -> Result<Vec<String>> {
    let mut source_labels = extract_cros_workon_sources(details)?;

    // Chromium packages need its source code.
    // TODO: Remove this hack.
    if details.inherited.contains("chromium-source") {
        // TODO: We need USE flags to add src-internal.
        source_labels.push("@chrome//:src".to_owned());
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
        source_labels.push("@chromite//:src".to_owned());
    }

    source_labels.sort();
    source_labels.dedup();

    Ok(source_labels)
}

fn fixup_local_sources(graph: &mut HashMap<PackageSlotKey, PackageData>) {
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

    let all_packages: Vec<String> = graph
        .values()
        .flat_map(|data| &data.local_sources)
        .filter_map(|label| label.strip_suffix(":src"))
        .sorted()
        .dedup()
        .map(|package| package.to_owned())
        .collect();

    let child_packages_map: BTreeMap<&str, Vec<&str>> = {
        let mut child_packages_map: BTreeMap<&str, Vec<&str>> = all_packages
            .iter()
            .map(|package| (package.as_str(), Vec::new()))
            .collect();
        let mut parent_packages_stack = Vec::<&str>::new();

        for current_package in all_packages.iter() {
            while let Some(last_parent_package) = parent_packages_stack.pop() {
                if current_package.starts_with(&format!("{}/", last_parent_package)) {
                    parent_packages_stack.push(last_parent_package);
                    break;
                }
            }

            for parent_package in parent_packages_stack.iter() {
                child_packages_map
                    .get_mut(*parent_package)
                    .unwrap()
                    .push(current_package);
            }

            if current_package != "//platform2" {
                parent_packages_stack.push(current_package);
            }
        }

        child_packages_map
    };

    for (_, data) in graph.iter_mut() {
        let local_sources = std::mem::take(&mut data.local_sources);
        let local_sources = local_sources
            .into_iter()
            .flat_map(|old_label| {
                let mut new_labels = vec![old_label.clone()];
                if let Some(old_package) = old_label.strip_suffix(":src") {
                    if let Some(packages) = child_packages_map.get(old_package) {
                        new_labels
                            .extend(packages.iter().map(|package| format!("{}:src", *package)));
                    }
                }
                new_labels
            })
            .sorted()
            .dedup()
            .collect();
        data.local_sources = local_sources;
    }
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

fn is_rust_source_package(details: &PackageDetails) -> bool {
    let is_rust_package = details.inherited.contains("cros-rust");
    let is_cros_workon_package = details.inherited.contains("cros-workon");
    let has_src_compile = match details.vars.get("HAS_SRC_COMPILE") {
        Some(BashValue::Scalar(s)) if s == "1" => true,
        _ => false,
    };

    is_rust_package && !is_cros_workon_package && !has_src_compile
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

    // Some Rust source packages have their dependencies only listed as DEPEND.
    // They also need to be listed as RDPEND so they get pulled in as transitive
    // deps.
    // TODO: Fix ebuilds and remove this hack.
    let runtime_deps = if is_rust_source_package(&details) {
        runtime_deps
            .into_iter()
            .chain(build_deps.clone().into_iter())
            .sorted()
            .dedup()
            .collect()
    } else {
        runtime_deps
    };

    let unresolved_post_deps =
        extract_package_dependencies(resolver, &*details, DependencyKind::Post).with_context(
            || {
                format!(
                    "Resolving post-time dependencies for {}-{}",
                    &details.package_name, &details.version
                )
            },
        )?;

    // Extract local source labels.
    let local_sources = extract_local_sources(&details).with_context(|| {
        format!(
            "Resolving local source dependencies for {}-{}",
            &details.package_name, &details.version
        )
    })?;

    // Extract remote source URIs.
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
                local_sources,
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
        local_sources: unresolved.local_sources,
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

    let mut resolution = resolution.into_inner().unwrap();

    fixup_local_sources(&mut resolution);

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
    /// Labels of Bazel ebuild_src targets this package depends on, e.g.
    /// "//platform/empty-project:src", "@chromite//:src".
    #[serde(rename = "localSrc")]
    local_src: Vec<String>,
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
                local_src: data.local_sources.clone(),
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
