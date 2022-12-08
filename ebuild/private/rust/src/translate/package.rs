// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};

use crate::{
    data::UseMap,
    dependency::{
        algorithm::{elide_use_conditions, simplify},
        package::{PackageAtomDependency, PackageBlock, PackageDependency},
        CompositeDependency, Dependency,
    },
    resolver::{FindBestPackageError, Resolver},
};

use super::parse_simplified_dependency;

fn simplify_dependency(
    deps: PackageDependency,
    use_map: &UseMap,
    resolver: &Resolver,
) -> Result<PackageDependency> {
    let deps = elide_use_conditions(deps, use_map);

    // Rewrite atoms.
    let deps = deps.try_map_tree_par(|dep| -> Result<PackageDependency> {
        match dep {
            Dependency::Leaf(atom) => {
                // Remove blocks.
                if atom.block() != PackageBlock::None {
                    return Ok(Dependency::new_constant(true));
                }

                // Remove provided packages.
                if let Some(_) = resolver.find_provided_packages(&atom).next() {
                    return Ok(Dependency::new_constant(true));
                }

                // Remove non-existent packages.
                match resolver.find_best_package(&atom) {
                    Err(FindBestPackageError::NotFound) => {
                        return Ok(Dependency::new_constant(false));
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

    Ok(simplify(deps))
}

pub fn translate_package_dependency(
    deps: PackageDependency,
    use_map: &UseMap,
    resolver: &Resolver,
) -> Result<Vec<PackageAtomDependency>> {
    let orig_deps = deps;
    let simplified_deps = simplify_dependency(orig_deps.clone(), use_map, resolver)?;

    parse_simplified_dependency(simplified_deps.clone()).with_context(|| {
        format!(
            "Failed to simplify dependencies\nOriginal: {}\nSimplified: {}",
            &orig_deps, &simplified_deps
        )
    })
}
