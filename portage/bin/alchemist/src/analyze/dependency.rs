// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::sync::Arc;

use anyhow::{Context, Result};
use itertools::Itertools;

use crate::{
    bash::vars::BashValue,
    data::UseMap,
    dependency::{
        algorithm::{elide_use_conditions, parse_simplified_dependency, simplify},
        package::{PackageBlock, PackageDependency},
        CompositeDependency, Dependency,
    },
    ebuild::PackageDetails,
    resolver::PackageResolver,
};

/// Analyzed package dependencies of a package. It is returned by
/// [`analyze_dependencies`].
///
/// This struct represents dependencies as lists of [`PackageDetails`] instead
/// of [`PackageDependency`] that can contain complex expressions such as
/// any-of.
#[derive(Clone, Debug)]
pub struct PackageDependencies {
    pub build_deps: Vec<Arc<PackageDetails>>,
    pub runtime_deps: Vec<Arc<PackageDetails>>,
    pub post_deps: Vec<Arc<PackageDetails>>,
    pub build_host_deps: Vec<Arc<PackageDetails>>,
    pub install_host_deps: Vec<Arc<PackageDetails>>,
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
    /// Build-time host tool dependencies, aka "BDEPEND" in Portage.
    /// `cross_compile` will be true when CBUILD != CHOST.
    BuildHost { cross_compile: bool },
    /// Install-time host tool dependencies, aka "IDEPEND" in Portage.
    /// `cross_compile` will be true when CBUILD != CHOST.
    InstallHost { cross_compile: bool },
}

/// Parses a dependency represented as [`PackageDependency`] that can contain
/// complex expressions such as any-of to a simple list of [`PackageDetails`].
fn parse_dependencies(
    deps: PackageDependency,
    use_map: &UseMap,
    resolver: &PackageResolver,
    allow_list: Option<&[&str]>,
) -> Result<Vec<Arc<PackageDetails>>> {
    let deps = elide_use_conditions(deps, use_map).unwrap_or_default();

    // Rewrite atoms.
    let deps = deps.try_map_tree_par(|dep| -> Result<PackageDependency> {
        match dep {
            Dependency::Leaf(atom) => {
                // Remove blocks.
                if atom.block() != PackageBlock::None {
                    return Ok(Dependency::new_constant(
                        true,
                        &format!("Package block {} is ignored", atom),
                    ));
                }

                // Remove packages not specified in the allow list.
                // This is a work around for EAPI < 7 packages that don't
                // support BDEPENDs.
                if let Some(allow_list) = allow_list {
                    if !allow_list.contains(&atom.package_name()) {
                        return Ok(Dependency::new_constant(
                            true,
                            &format!("Package {} is not in allowed list", atom),
                        ));
                    }
                }

                // Remove provided packages.
                if resolver.find_provided_packages(&atom).next().is_some() {
                    return Ok(Dependency::new_constant(
                        true,
                        &format!("Package {} is in package.provided", atom),
                    ));
                }

                // Remove non-existent packages.
                match resolver.find_best_package_dependency(use_map, &atom) {
                    Ok(result) => {
                        if result.is_none() {
                            return Ok(Dependency::new_constant(
                                false,
                                &format!("No package satisfies {}", atom),
                            ));
                        };
                    }
                    Err(err) => {
                        return Ok(Dependency::new_constant(
                            false,
                            &format!("Error matching {}: {:?}", atom, err),
                        ));
                    }
                }

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

    let atoms = parse_simplified_dependency(deps)?;

    atoms
        .into_iter()
        .map(|atom| {
            Ok(
                resolver
                    .find_best_package_dependency(use_map, &atom)?
                    .expect("package to exist"), // missing packages were filtered above
            )
        })
        .collect::<Result<Vec<_>>>()
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
        // When cross compiling `dev-libs/nss`, it requires `dev-libs/nss` to be
        // installed on the build host. We can't add `dev-libs/nss` as a BDEPEND
        // to the ebuild because that would cause a circular dependency when
        // building for the host.
        // See: https://bugs.gentoo.org/759127
        (
            "dev-libs/nss",
            DependencyKind::BuildHost {
                cross_compile: true,
            },
        ) => "dev-libs/nss",
        // dev-libs/nss needs to run the `shlibsign` binary when installing.
        // When cross-compiling that means we need need to use the build host's
        // `shlibsign`.
        (
            "dev-libs/nss",
            DependencyKind::InstallHost {
                cross_compile: true,
            },
        ) => "dev-libs/nss",
        _ => "",
    }
}

fn extract_dependencies(
    details: &PackageDetails,
    kind: DependencyKind,
    resolver: &PackageResolver,
    allow_list: Option<&[&str]>,
) -> Result<Vec<Arc<PackageDetails>>> {
    let var_name = match kind {
        DependencyKind::Build => "DEPEND",
        DependencyKind::Run => "RDEPEND",
        DependencyKind::Post => "PDEPEND",
        DependencyKind::BuildHost { .. } => "BDEPEND",
        DependencyKind::InstallHost { .. } => "IDEPEND",
    };

    let raw_deps = details.vars.get_scalar_or_default(var_name)?;

    let raw_extra_deps = get_extra_dependencies(details, kind);

    let joined_raw_deps = format!("{} {}", raw_deps, raw_extra_deps);
    let deps = joined_raw_deps.parse::<PackageDependency>()?;

    parse_dependencies(deps, &details.use_map, resolver, allow_list)
}

// TODO: Remove this hack.
fn is_rust_source_package(details: &PackageDetails) -> bool {
    let is_rust_package = details.inherited.contains("cros-rust");
    let is_cros_workon_package = details.inherited.contains("cros-workon");
    let has_src_compile = matches!(
        details.vars.hash_map().get("HAS_SRC_COMPILE"),
        Some(BashValue::Scalar(s)) if s == "1");

    is_rust_package && !is_cros_workon_package && !has_src_compile
}

// We don't want to open the flood gates and pull in ALL DEPENDs
// because there are only a handful that are actually BDEPENDs.
// We keep a hand curated list of packages that are known to be
// BDEPENDs. Ideally we upgrade all ebuilds to EAPI7 and delete this
// block, but that's a lot of work.
static DEPEND_AS_BDEPEND_ALLOW_LIST: [&str; 13] = [
    "app-portage/elt-patches",
    "dev-util/cmake",
    "dev-util/meson",
    "dev-util/meson-format-array",
    "dev-util/ninja",
    "sys-devel/autoconf",
    "sys-devel/autoconf-archive",
    "sys-devel/automake",
    "sys-devel/bison",
    "sys-devel/flex",
    "sys-devel/gnuconfig",
    "sys-devel/libtool",
    "sys-devel/make",
];

/// Analyzes ebuild variables and returns [`PackageDependencies`] containing
/// its dependencies as a list of [`PackageDetails`].
pub fn analyze_dependencies(
    details: &PackageDetails,
    cross_compile: bool,
    host_resolver: Option<&PackageResolver>,
    target_resolver: &PackageResolver,
) -> Result<PackageDependencies> {
    let build_deps = extract_dependencies(details, DependencyKind::Build, target_resolver, None)
        .with_context(|| {
            format!(
                "Resolving build-time dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?;

    let runtime_deps = extract_dependencies(details, DependencyKind::Run, target_resolver, None)
        .with_context(|| {
            format!(
                "Resolving runtime dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?;

    let build_host_deps = if let Some(host_resolver) = host_resolver {
        if details.supports_bdepend() {
            extract_dependencies(
                details,
                DependencyKind::BuildHost { cross_compile },
                host_resolver,
                None,
            )
        } else {
            // We need to apply the allow list filtering during dependency
            // evaluation instead of post-dependency evaluation because
            // there are dependencies that we can't satisfy using the host
            // resolver. i.e. `libchrome[cros_debug=]`.
            extract_dependencies(
                details,
                DependencyKind::Build,
                host_resolver,
                Some(&DEPEND_AS_BDEPEND_ALLOW_LIST),
            )
        }
        .with_context(|| {
            format!(
                "Resolving build-time host dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?
    } else {
        vec![]
    };

    let install_host_deps = if let Some(host_resolver) = host_resolver {
        extract_dependencies(
            details,
            DependencyKind::InstallHost { cross_compile },
            host_resolver,
            None,
        )
        .with_context(|| {
            format!(
                "Resolving install-time host dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?
    } else {
        vec![]
    };

    // Some Rust source packages have their dependencies only listed as DEPEND.
    // They also need to be listed as RDPEND so they get pulled in as transitive
    // deps.
    // TODO: Fix ebuilds and remove this hack.
    let runtime_deps = if is_rust_source_package(details) {
        runtime_deps
            .into_iter()
            .chain(build_deps.clone().into_iter())
            .sorted_by(|a, b| {
                a.package_name
                    .cmp(&b.package_name)
                    .then(a.version.cmp(&b.version))
            })
            .dedup_by(|a, b| a.package_name == b.package_name && a.version == b.version)
            .collect()
    } else {
        runtime_deps
    };

    let post_deps = extract_dependencies(details, DependencyKind::Post, target_resolver, None)
        .with_context(|| {
            format!(
                "Resolving post-time dependencies for {}-{}",
                &details.package_name, &details.version
            )
        })?;

    Ok(PackageDependencies {
        build_deps,
        runtime_deps,
        post_deps,
        build_host_deps,
        install_host_deps,
    })
}
