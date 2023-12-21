// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod direct;

use anyhow::Result;

use crate::{ebuild::PackageDetails, resolver::PackageResolver};

use self::direct::{analyze_direct_dependencies, DirectDependencies};

pub type PackageDependencies = DirectDependencies;

/// Analyzes dependencies of the given package.
pub fn analyze_dependencies(
    details: &PackageDetails,
    cross_compile: bool,
    host_resolver: &PackageResolver,
    target_resolver: &PackageResolver,
) -> Result<PackageDependencies> {
    analyze_direct_dependencies(details, cross_compile, host_resolver, target_resolver)
}
