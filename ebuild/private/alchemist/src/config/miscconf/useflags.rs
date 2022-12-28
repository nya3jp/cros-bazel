// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use itertools::Itertools;
use std::{fs::read_to_string, path::Path};

use crate::{
    config::{ConfigNode, ConfigNodeValue, UseUpdate, UseUpdateFilter, UseUpdateKind},
    dependency::package::PackageAtomDependency,
};

fn load_wildcard_use_config(
    source: &Path,
    kind: UseUpdateKind,
    stable_only: bool,
) -> Result<Vec<ConfigNode>> {
    if !source.exists() {
        return Ok(Vec::new());
    }

    if source.is_dir() {
        let mut names = Vec::new();
        for entry in source.read_dir()? {
            names.push(entry?.file_name());
        }
        names.sort();

        let mut nodes = Vec::<ConfigNode>::new();
        for name in names {
            let new_source = source.join(name);
            nodes.extend(
                load_wildcard_use_config(&new_source, kind, stable_only)
                    .with_context(|| format!("Loading {}", new_source.to_string_lossy()))?,
            );
        }
        return Ok(nodes);
    }

    let contents = read_to_string(source)?;

    let mut updates = Vec::<UseUpdate>::new();

    for tokens in contents
        .split("\n")
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("#"))
    {
        updates.push(UseUpdate {
            kind,
            filter: UseUpdateFilter {
                atom: None,
                stable_only,
            },
            use_tokens: tokens.to_owned(),
        })
    }

    Ok(vec![ConfigNode {
        source: source.to_owned(),
        value: ConfigNodeValue::Uses(updates),
    }])
}

fn load_package_use_config(
    source: &Path,
    kind: UseUpdateKind,
    stable_only: bool,
) -> Result<Vec<ConfigNode>> {
    if !source.exists() {
        return Ok(Vec::new());
    }

    if source.is_dir() {
        let mut names = Vec::new();
        for entry in source.read_dir()? {
            names.push(entry?.file_name());
        }
        names.sort();

        let mut nodes = Vec::<ConfigNode>::new();
        for name in names {
            let new_source = source.join(name);
            nodes.extend(
                load_package_use_config(&new_source, kind, stable_only)
                    .with_context(|| format!("Loading {}", new_source.to_string_lossy()))?,
            );
        }
        return Ok(nodes);
    }

    let contents = read_to_string(source)?;

    let mut updates = Vec::<UseUpdate>::new();

    for line in contents
        .split("\n")
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("#"))
    {
        let (raw_atom, tokens) = line
            .trim()
            .split_once(|c: char| c.is_ascii_whitespace())
            .unwrap_or((line, ""));
        let atom = raw_atom.parse::<PackageAtomDependency>()?;
        updates.push(UseUpdate {
            kind,
            filter: UseUpdateFilter {
                atom: Some(atom),
                stable_only,
            },
            use_tokens: tokens.to_owned(),
        })
    }

    Ok(vec![ConfigNode {
        source: source.to_owned(),
        value: ConfigNodeValue::Uses(updates),
    }])
}

pub fn load_use_configs(dir: &Path) -> Result<Vec<ConfigNode>> {
    Ok([
        // Set
        load_package_use_config(&dir.join("package.use"), UseUpdateKind::Set, false)?,
        // Mask
        load_wildcard_use_config(&dir.join("use.mask"), UseUpdateKind::Mask, false)?,
        load_wildcard_use_config(&dir.join("use.stable.mask"), UseUpdateKind::Mask, true)?,
        load_package_use_config(&dir.join("package.use.mask"), UseUpdateKind::Mask, false)?,
        load_package_use_config(
            &dir.join("package.use.stable.mask"),
            UseUpdateKind::Mask,
            true,
        )?,
        // Force
        load_wildcard_use_config(&dir.join("use.force"), UseUpdateKind::Force, false)?,
        load_wildcard_use_config(&dir.join("use.stable.force"), UseUpdateKind::Force, true)?,
        load_package_use_config(&dir.join("package.use.force"), UseUpdateKind::Force, false)?,
        load_package_use_config(
            &dir.join("package.use.stable.force"),
            UseUpdateKind::Force,
            true,
        )?,
    ]
    .into_iter()
    .concat())
}
