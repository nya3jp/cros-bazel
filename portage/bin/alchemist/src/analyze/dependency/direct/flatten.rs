// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::sync::Arc;

use anyhow::Result;
use itertools::Itertools;

use crate::{
    data::UseMap,
    dependency::{
        algorithm::{elide_use_conditions, parse_simplified_dependency, simplify},
        package::{PackageBlock, PackageDependency},
        CompositeDependency, Dependency,
    },
    ebuild::PackageDetails,
    resolver::PackageResolver,
};

/// Flattens a dependency represented as [`PackageDependency`] that can contain
/// complex expressions such as any-of to a simple list of [`PackageDetails`].
pub fn flatten_dependencies(
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
                // If all children are false, concat the error messages since
                // they are helpful for debugging what happened.
                if children.iter().all(|c| match c {
                    Dependency::Composite(composite) => match &**composite {
                        CompositeDependency::Constant { value, .. } => !value,
                        _ => false,
                    },
                    _ => false,
                }) {
                    let result = children
                        .iter()
                        .filter_map(|c| match c {
                            Dependency::Composite(composite) => match &**composite {
                                CompositeDependency::Constant { reason, .. } => Some(reason),
                                _ => None,
                            },
                            _ => None,
                        })
                        .join(", ");
                    Dependency::new_constant(false, &format!("any-of ( {result} )"))
                } else {
                    children.into_iter().next().unwrap()
                }
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
