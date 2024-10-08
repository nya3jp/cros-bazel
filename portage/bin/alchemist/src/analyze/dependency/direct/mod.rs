// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod flatten;
mod hacks;
mod slot;

use std::sync::Arc;

use anyhow::{Context, Result};
use itertools::Itertools;

use crate::{
    data::UseMap, dependency::package::PackageDependency, ebuild::PackageDetails,
    resolver::PackageResolver,
};

use self::{
    flatten::flatten_dependencies,
    hacks::{get_extra_dependencies, is_rust_source_package, DEPEND_AS_BDEPEND_ALLOW_LIST},
    slot::rewrite_subslot_deps,
};

/// Analyzed direct dependencies of a package. It is returned by [`analyze_direct_dependencies`].
///
/// This struct represents dependencies as lists of [`PackageDetails`] instead of
/// [`PackageDependency`] that can contain complex expressions such as any-of.
#[derive(Clone, Debug)]
pub struct DirectDependencies {
    /// Target packages to install before building the package, aka DEPEND.
    pub build_target: Vec<Arc<PackageDetails>>,

    /// Target packages to install before running tests of the package.
    pub test_target: Vec<Arc<PackageDetails>>,

    /// Target packages to install before making the package usable, aka RDEPEND.
    pub run_target: Vec<Arc<PackageDetails>>,

    /// Target packages to install to make the package usable (the order does not matter),
    /// aka PDEPEND.
    pub post_target: Vec<Arc<PackageDetails>>,

    /// Host packages to install before building the package, aka BDEPEND.
    pub build_host: Vec<Arc<PackageDetails>>,

    /// Host packages to install before installing the package, aka IDEPEND.
    pub install_host: Vec<Arc<PackageDetails>>,
}

impl DirectDependencies {
    pub fn get(&self, kind: DependencyKind) -> &Vec<Arc<PackageDetails>> {
        match kind {
            DependencyKind::BuildTarget => &self.build_target,
            DependencyKind::RunTarget => &self.run_target,
            DependencyKind::PostTarget => &self.post_target,
            DependencyKind::BuildHost => &self.build_host,
            DependencyKind::InstallHost => &self.install_host,
        }
    }
}

/// A package's *DEPEND expressions.
///
/// Contains the value of the *DEPEND keys that would be written into the
/// binpkg's XPAK. If an atom contained a sub-slot rebuild operator (:=), the
/// atom is rewritten to contain the best matching slot/sub-slot. This is used
/// to aid portage in determining when a package should be rebuilt.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct DependencyExpressions {
    /// Target packages to install before building the package, aka DEPEND.
    pub build_target: String,

    /// Target packages to install before making the package usable, aka RDEPEND.
    pub run_target: String,

    /// Target packages to install to make the package usable, aka PDEPEND.
    pub post_target: String,

    /// Host packages to install before building the package, aka BDEPEND.
    pub build_host: String,

    /// Host packages to install before installing the package, aka IDEPEND.
    pub install_host: String,
}

/// Represents a package dependency type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DependencyKind {
    /// Build-time dependencies, aka "DEPEND" in Portage.
    BuildTarget,
    /// Run-time dependencies, aka "RDEPEND" in Portage.
    RunTarget,
    /// Post-time dependencies, aka "PDEPEND" in Portage.
    PostTarget,
    /// Build-time host tool dependencies, aka "BDEPEND" in Portage.
    BuildHost,
    /// Install-time host tool dependencies, aka "IDEPEND" in Portage.
    InstallHost,
}

// TODO(b:299056510): Consider removing 4-argument variant of this function.
fn extract_dependencies(
    details: &PackageDetails,
    kind: DependencyKind,
    cross_compile: bool,
    resolver: &PackageResolver,
    allow_list: Option<&[&str]>,
) -> Result<(Vec<Arc<PackageDetails>>, String)> {
    extract_dependencies_use(
        details,
        &details.use_map,
        kind,
        cross_compile,
        resolver,
        allow_list,
    )
}

fn extract_dependencies_use(
    details: &PackageDetails,
    use_map: &UseMap,
    kind: DependencyKind,
    cross_compile: bool,
    resolver: &PackageResolver,
    allow_list: Option<&[&str]>,
) -> Result<(Vec<Arc<PackageDetails>>, String)> {
    let var_name = match kind {
        DependencyKind::BuildTarget => Some("DEPEND"),
        DependencyKind::RunTarget => Some("RDEPEND"),
        DependencyKind::PostTarget => Some("PDEPEND"),
        DependencyKind::BuildHost => Some("BDEPEND"),
        DependencyKind::InstallHost => details.supports_idepend().then_some("IDEPEND"),
    };

    let raw_deps = var_name.map_or(Ok(""), |var_name| {
        details.metadata.vars.get_scalar_or_default(var_name)
    })?;

    let raw_extra_deps = get_extra_dependencies(details, kind, cross_compile);

    let joined_raw_deps = format!("{} {}", raw_deps, raw_extra_deps);
    let deps = joined_raw_deps.parse::<PackageDependency>()?;

    let dep_list = flatten_dependencies(deps.clone(), use_map, resolver, allow_list)?;

    let expression = rewrite_subslot_deps(deps, use_map, resolver)?;

    Ok((dep_list, expression))
}

/// Analyzes ebuild variables to determine direct dependencies of a package.
pub fn analyze_direct_dependencies(
    details: &PackageDetails,
    cross_compile: bool,
    host_resolver: &PackageResolver,
    target_resolver: &PackageResolver,
) -> Result<(DirectDependencies, DependencyExpressions)> {
    let (build_target_deps, build_target_expr) = extract_dependencies(
        details,
        DependencyKind::BuildTarget,
        cross_compile,
        target_resolver,
        None,
    )
    .with_context(|| {
        format!(
            "Resolving build-time dependencies for {}-{}",
            &details.as_basic_data().package_name,
            &details.as_basic_data().version
        )
    })?;

    let (test_target_deps, _test_target_expr) = if details.use_map.contains_key("test") {
        let mut test_use_map = details.use_map.clone();
        test_use_map.insert("test".into(), true);
        // Hack: We often (more than 100 packages) fail to resolve test-only
        // dependencies. This happens when a package pulls something that
        // cannot be found (for example, sys-apps/dbus depends on
        // x11-base/xorg-server) or requires a package compiled with a flag
        // (chromeos-base/libhwsec:=[test?]). In this case we fall back on
        // build_deps.
        // TODO(b:299056510): Emit always_fail if there are unresolved deps.
        let test_deps_result = extract_dependencies_use(
            details,
            &test_use_map,
            DependencyKind::BuildTarget,
            cross_compile,
            target_resolver,
            None,
        );
        test_deps_result.unwrap_or_else(|_| (build_target_deps.clone(), build_target_expr.clone()))
    } else {
        // The ebuild does not care about use flag, so test deps are the same
        // as build deps.
        (build_target_deps.clone(), build_target_expr.clone())
    };

    let (run_target_deps, run_target_expr) = extract_dependencies(
        details,
        DependencyKind::RunTarget,
        cross_compile,
        target_resolver,
        None,
    )
    .with_context(|| {
        format!(
            "Resolving runtime dependencies for {}-{}",
            &details.as_basic_data().package_name,
            &details.as_basic_data().version
        )
    })?;

    let (build_host_deps, build_host_expr) = {
        // We query BDEPEND regardless of EAPI because we want our overrides
        // from `get_extra_dependencies` to allow specifying a BDEPEND even
        // if the EAPI doesn't support it.
        let (mut build_host_deps, build_host_expr) = extract_dependencies(
            details,
            DependencyKind::BuildHost,
            cross_compile,
            host_resolver,
            None,
        )
        .with_context(|| {
            format!(
                "Resolving build-time host dependencies for {}-{}",
                &details.as_basic_data().package_name,
                &details.as_basic_data().version
            )
        })?;

        if !details.supports_bdepend() {
            // We need to apply the allow list filtering during dependency
            // evaluation instead of post-dependency evaluation because
            // there are dependencies that we can't satisfy using the host
            // resolver. i.e. `libchrome[cros_debug=]`.
            let build_deps_for_host = extract_dependencies(
                details,
                DependencyKind::BuildTarget,
                cross_compile,
                host_resolver,
                Some(&DEPEND_AS_BDEPEND_ALLOW_LIST),
            )
            .with_context(|| {
                format!(
                    "Resolving build-time dependencies as host dependencies for {}-{}",
                    &details.as_basic_data().package_name,
                    &details.as_basic_data().version
                )
            })?;

            for package_details in build_deps_for_host.0 {
                if !build_host_deps.iter().any(|a| {
                    a.as_basic_data().ebuild_path == package_details.as_basic_data().ebuild_path
                }) {
                    build_host_deps.push(package_details);
                }
            }
        }

        (build_host_deps, build_host_expr)
    };

    let (install_host_deps, install_host_expr) = extract_dependencies(
        details,
        DependencyKind::InstallHost,
        cross_compile,
        host_resolver,
        None,
    )
    .with_context(|| {
        format!(
            "Resolving install-time host dependencies for {}-{}",
            &details.as_basic_data().package_name,
            &details.as_basic_data().version
        )
    })?;

    // Some Rust source packages have their dependencies only listed as DEPEND.
    // They also need to be listed as RDPEND so they get pulled in as transitive
    // deps.
    // TODO: Fix ebuilds and remove this hack.
    let run_target_deps = if is_rust_source_package(details) {
        run_target_deps
            .into_iter()
            .chain(build_target_deps.clone())
            .sorted_by(|a, b| {
                a.as_basic_data()
                    .package_name
                    .cmp(&b.as_basic_data().package_name)
                    .then(a.as_basic_data().version.cmp(&b.as_basic_data().version))
            })
            .dedup_by(|a, b| {
                a.as_basic_data().package_name == b.as_basic_data().package_name
                    && a.as_basic_data().version == b.as_basic_data().version
            })
            .collect()
    } else {
        run_target_deps
    };

    let (post_target_deps, post_target_expr) = extract_dependencies(
        details,
        DependencyKind::PostTarget,
        cross_compile,
        target_resolver,
        None,
    )
    .with_context(|| {
        format!(
            "Resolving post-time dependencies for {}-{}",
            &details.as_basic_data().package_name,
            &details.as_basic_data().version
        )
    })?;

    Ok((
        DirectDependencies {
            build_target: build_target_deps,
            test_target: test_target_deps,
            run_target: run_target_deps,
            post_target: post_target_deps,
            build_host: build_host_deps,
            install_host: install_host_deps,
        },
        DependencyExpressions {
            build_target: build_target_expr,
            run_target: run_target_expr,
            post_target: post_target_expr,
            build_host: build_host_expr,
            install_host: install_host_expr,
        },
    ))
}
