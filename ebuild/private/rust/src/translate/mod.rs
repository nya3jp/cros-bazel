// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::fmt::Display;
use std::hash::Hash;

use crate::dependency::{CompositeDependency, Dependency};
use anyhow::{bail, Result};
use itertools::Itertools;

pub mod package;

fn parse_simplified_dependency<L: Clone + Display + Eq + Hash>(
    deps: Dependency<L>,
) -> Result<Vec<L>> {
    match deps {
        Dependency::Leaf(atom) => Ok(vec![atom]),
        Dependency::Composite(composite) => match *composite {
            CompositeDependency::AllOf { children } => {
                let atoms = children
                    .into_iter()
                    .map(|child| match child {
                        Dependency::Leaf(atom) => Ok(atom),
                        _ => bail!(
                            "Found a non-atom dependency after simplification: {}",
                            child
                        ),
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(atoms.into_iter().unique().collect())
            }
            other => bail!(
                "Found a non-atom dependency after simplification: {}",
                Dependency::new_composite(other)
            ),
        },
    }
}
