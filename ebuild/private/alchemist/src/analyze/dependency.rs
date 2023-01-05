// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use itertools::Itertools;

use crate::{
    bash::BashValue,
    data::UseMap,
    dependency::{
        algorithm::{elide_use_conditions, parse_simplified_dependency, simplify},
        package::{PackageAtomDependency, PackageBlock, PackageDependency},
        CompositeDependency, Dependency,
    },
    ebuild::PackageDetails,
    resolver::{FindBestPackageError, PackageResolver},
};

/// Analyzed package dependencies of a package. It is returned by
/// [`analyze_dependencies`].
///
/// This struct represents dependencies as lists of [`PackageAtomDependency`]
/// instead of [`PackageDependency`] that can contain complex expressions such
/// as any-of.
#[derive(Clone, Debug)]
pub struct PackageDependencies {
    pub build_deps: Vec<PackageAtomDependency>,
    pub runtime_deps: Vec<PackageAtomDependency>,
    pub post_deps: Vec<PackageAtomDependency>,
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

/// Parses a depenendency represented as [`PackageDependency`] that can contain
/// complex expressions such as any-of to a simple list of
/// [`PackageAtomDependency`].
fn parse_dependencies(
    deps: PackageDependency,
    use_map: &UseMap,
    resolver: &PackageResolver,
) -> Result<Vec<PackageAtomDependency>> {
    let deps = elide_use_conditions(deps, use_map).unwrap_or_default();

    // Rewrite atoms.
    let deps = deps.try_map_tree_par(|dep| -> Result<PackageDependency> {
        match dep {
            Dependency::Leaf(atom) => {
                // Remove blocks.
                if atom.block() != PackageBlock::None {
                    return Ok(Dependency::new_constant(
                        true,
                        &format!("Package block {} is ignored", atom.to_string()),
                    ));
                }

                // Remove provided packages.
                if let Some(_) = resolver.find_provided_packages(&atom).next() {
                    return Ok(Dependency::new_constant(
                        true,
                        &format!("Package {} is in package.provided", atom.to_string()),
                    ));
                }

                // Remove non-existent packages.
                match resolver.find_best_package(&atom) {
                    Err(FindBestPackageError::NotFound) => {
                        return Ok(Dependency::new_constant(
                            false,
                            &format!("No package satisfies {}", atom.to_string()),
                        ));
                    }
                    res => {
                        res?;
                    }
                };

                Ok(Dependency::Leaf(atom))
            }
            _ => Ok(dep),
        }
    })?;

    let deps = simplify(deps);

    // Resolve any-of dependencies by picking the first item.
    let deps = deps.map_tree_par(|dep| match dep {
        Dependency::Composite(composite) => match *composite {
            CompositeDependency::AnyOf { children } if !children.is_empty() => {
                children.into_iter().next().unwrap()
            }
            other => Dependency::new_composite(other),
        },
        other => other,
    });

    let deps = simplify(deps);

    parse_simplified_dependency(deps.clone())
}

// TODO: Remove this hack.
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

fn extract_dependencies(
    details: &PackageDetails,
    kind: DependencyKind,
    resolver: &PackageResolver,
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

    parse_dependencies(deps, &details.use_map, resolver)
}

// TODO: Remove this hack.
fn is_rust_source_package(details: &PackageDetails) -> bool {
    let is_rust_package = details.inherited.contains("cros-rust");
    let is_cros_workon_package = details.inherited.contains("cros-workon");
    let has_src_compile = match details.vars.get("HAS_SRC_COMPILE") {
        Some(BashValue::Scalar(s)) if s == "1" => true,
        _ => false,
    };

    is_rust_package && !is_cros_workon_package && !has_src_compile
}

/// Analyzes ebuild variables and returns [`PackageDependencies`] containing
/// its dependencies as a list of [`PackageAtomDependency`].
pub fn analyze_dependencies(
    details: &PackageDetails,
    resolver: &PackageResolver,
) -> Result<PackageDependencies> {
    let build_deps = extract_dependencies(&*details, DependencyKind::Build, resolver)
        .with_context(|| {
            format!(
                "Resolving build-time dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?;

    let runtime_deps = extract_dependencies(&*details, DependencyKind::Run, resolver)
        .with_context(|| {
            format!(
                "Resolving runtime dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?;

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

    let post_deps =
        extract_dependencies(&*details, DependencyKind::Post, resolver).with_context(|| {
            format!(
                "Resolving post-time dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?;

    Ok(PackageDependencies {
        build_deps,
        runtime_deps,
        post_deps,
    })
}
