// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{Arc, Mutex},
};

use alchemist::{
    analyze::{
        dependency::analyze_dependencies,
        source::{analyze_sources, fixup_sources, PackageSources},
    },
    data::PackageSlotKey,
    dependency::package::PackageAtomDependency,
    ebuild::PackageDetails,
    resolver::PackageResolver,
};
use anyhow::{bail, Result};
use itertools::Itertools;
use rayon::prelude::*;
use rpds::{HashTrieSetSync, VectorSync};
use serde::{Deserialize, Serialize};

/// Package selected by [`select_packages`].
///
/// It contains [`PackageDetails`] as well as resolved dependency info.
#[derive(Clone, Debug)]
struct SelectedPackage {
    details: Arc<PackageDetails>,
    build_deps: Vec<PackageSlotKey>,
    runtime_deps: Vec<PackageSlotKey>,
    post_deps: Vec<PackageSlotKey>,
}

/// A set of packages selected by [`select_packages`].
type SelectedPackageMap = HashMap<PackageSlotKey, SelectedPackage>;

/// Similar to [`SelectedPackage`], but post-dependencies are still unresolved.
/// This is used in the middle of computing the package selection.
#[derive(Clone, Debug)]
struct UnresolvedPackage {
    details: Arc<PackageDetails>,
    build_deps: Vec<PackageSlotKey>,
    runtime_deps: Vec<PackageSlotKey>,
    unresolved_post_deps: Vec<PackageAtomDependency>,
}

/// Similar to [`SelectedPackageMap`], but its values are [`UnresolvedPackage`].
/// This is used in the middle of computing the package selection.
type UnresolvedPackageMap = HashMap<PackageSlotKey, UnresolvedPackage>;

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

fn select_package(
    selection: &Mutex<UnresolvedPackageMap>,
    path: &SearchPath,
    atom: &PackageAtomDependency,
    resolver: &PackageResolver,
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

    // Analyze dependencies.
    let unresolved_deps = analyze_dependencies(&*details, resolver)?;

    let build_deps =
        select_packages_parallel(selection, &path, unresolved_deps.build_deps, resolver)?;
    let runtime_deps =
        select_packages_parallel(selection, &path, unresolved_deps.runtime_deps, resolver)?;
    let unresolved_post_deps = unresolved_deps.post_deps;

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
            let new_package = UnresolvedPackage {
                details,
                build_deps,
                runtime_deps,
                unresolved_post_deps,
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

fn select_packages_parallel(
    selection: &Mutex<UnresolvedPackageMap>,
    path: &SearchPath,
    atoms: impl IntoParallelIterator<Item = PackageAtomDependency>,
    resolver: &PackageResolver,
) -> Result<Vec<PackageSlotKey>> {
    Ok(atoms
        .into_par_iter()
        .map(|atom| select_package(selection, &path.clone(), &atom, resolver))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .sorted()
        .dedup()
        .collect_vec())
}

fn resolve_package(
    selection: &Mutex<UnresolvedPackageMap>,
    path: &SearchPath,
    unresolved: UnresolvedPackage,
    resolver: &PackageResolver,
) -> Result<SelectedPackage> {
    let post_deps =
        select_packages_parallel(selection, path, unresolved.unresolved_post_deps, resolver)?;
    Ok(SelectedPackage {
        details: unresolved.details,
        build_deps: unresolved.build_deps,
        runtime_deps: unresolved.runtime_deps,
        post_deps,
    })
}

/// Searches the dependency graph to find all packages depended by the specified
/// starting packages transitively.
///
/// The returned package set is keyed by [`PackageSlotKey`], and its values are
/// [`Package`] that contain [`PackageDetails`] as well as resolved dependencies
/// to other packages.
fn select_packages(
    starts: impl IntoIterator<Item = PackageAtomDependency>,
    resolver: &PackageResolver,
) -> Result<SelectedPackageMap> {
    let starts = starts.into_iter().collect_vec();

    let path = SearchPath::new();
    let selection: Mutex<UnresolvedPackageMap> = Default::default();
    let resolution: Mutex<SelectedPackageMap> = Default::default();

    select_packages_parallel(&selection, &path, starts, resolver)?;

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

            let resolved = resolve_package(&selection, &path, unresolved, resolver)?;

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

struct Package {
    selected: SelectedPackage,
    sources: PackageSources,
}

type PackageMap = HashMap<PackageSlotKey, Package>;

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

fn compute_label_map(graph: &PackageMap) -> HashMap<PackageSlotKey, String> {
    graph
        .iter()
        .map(|(key, package)| {
            let ebuild_path = package.selected.details.ebuild_path.to_string_lossy();
            let relative_ebuild_path = ebuild_path.split("/src/").last().unwrap();
            let relative_ebuild_components = relative_ebuild_path.split('/').collect_vec();
            let label = format!(
                "//{}:{}",
                relative_ebuild_components[..relative_ebuild_components.len() - 1].join("/"),
                &package.selected.details.slot.main
            );
            (key.clone(), label)
        })
        .collect()
}

/// The entry point of "dump-deps" subcommand.
pub fn dump_deps_main(
    resolver: &PackageResolver,
    starts: Vec<PackageAtomDependency>,
) -> Result<()> {
    let selected_packages = select_packages(starts, resolver)?;

    // Extract source info in parallel.
    let packages = {
        let mut packages = selected_packages
            .into_par_iter()
            .map(|(key, selected)| {
                let sources = analyze_sources(&selected.details)?;
                Ok((key, Package { selected, sources }))
            })
            .collect::<Result<PackageMap>>()?;
        fixup_sources(packages.values_mut().map(|package| &mut package.sources));
        packages
    };

    let label_map = compute_label_map(&packages);

    let data: BTreeMap<String, PackageInfo> = packages
        .iter()
        .map(|(key, package)| {
            let info = PackageInfo {
                name: package.selected.details.package_name.clone(),
                main_slot: package.selected.details.slot.main.clone(),
                ebuild_path: package
                    .selected
                    .details
                    .ebuild_path
                    .strip_prefix("/mnt/host/source/src")
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                version: package.selected.details.version.to_string(),
                build_deps: package
                    .selected
                    .build_deps
                    .iter()
                    .map(|key| label_map.get(key).unwrap().clone())
                    .sorted()
                    .collect(),
                runtime_deps: package
                    .selected
                    .runtime_deps
                    .iter()
                    .map(|key| label_map.get(key).unwrap().clone())
                    .sorted()
                    .collect(),
                post_deps: package
                    .selected
                    .post_deps
                    .iter()
                    .map(|key| label_map.get(key).unwrap().clone())
                    .sorted()
                    .collect(),
                local_src: package.sources.local_sources.clone(),
                src_uris: package
                    .sources
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
