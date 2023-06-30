// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use itertools::Itertools;
use std::{fs::read_to_string, path::Path, path::PathBuf};

use crate::{
    config::{AcceptKeywordsUpdate, ConfigNode, ConfigNodeValue},
    dependency::package::PackageAtom,
};

fn load_accept_keywords_configs_internal(source: PathBuf) -> Result<Vec<ConfigNode>> {
    // Return empty result if the path doesn't exist.
    if !source.try_exists()? {
        return Ok(Vec::new());
    }

    // Load child files and directories if it's a directory.
    if source.is_dir() {
        let mut names = source
            .read_dir()?
            .map(|entry| Ok(entry?.file_name()))
            .collect::<Result<Vec<_>>>()?;
        names.sort();

        return names
            .into_iter()
            .map(|name| {
                let new_source = source.join(name);
                load_accept_keywords_configs_internal(new_source)
                    .with_context(|| format!("Failed to load {}", source.display()))
            })
            .flatten_ok()
            .collect();
    }

    // Load a file.
    let updates = read_to_string(&source)?
        .split('\n')
        .map(|line| line.trim())
        .filter(|line| {
            // Filter empty lines and comments.
            !line.is_empty() && !line.starts_with('#')
        })
        .enumerate()
        .map(|(lineno, line)| {
            // A line consists of an atom followed by zero or more keywords.
            let (raw_atom, tokens) = line
                .trim()
                .split_once(|c: char| c.is_ascii_whitespace())
                .unwrap_or((line, ""));
            let atom = raw_atom.parse::<PackageAtom>().with_context(|| {
                format!("Failed to load {}: line {}", source.display(), lineno + 1)
            })?;
            Ok(AcceptKeywordsUpdate {
                atom,
                accept_keywords: tokens.to_owned(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(vec![ConfigNode {
        sources: vec![source],
        value: ConfigNodeValue::AcceptKeywords(updates),
    }])
}

/// Loads package.accept_keywords in the specified directory. If it's a file, just loads the file.
/// If it's a directory, loads its all descendant directories and files.
pub fn load_accept_keywords_configs(dir: &Path) -> Result<Vec<ConfigNode>> {
    load_accept_keywords_configs_internal(dir.join("package.accept_keywords"))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::testutils::write_files;

    use super::*;

    #[test]
    fn test_load_accept_keywords_configs_no_file() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        let nodes = load_accept_keywords_configs(dir)?;
        assert_eq!(Vec::<ConfigNode>::new(), nodes);
        Ok(())
    }

    #[test]
    fn test_load_accept_keywords_configs_load_file() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(dir, [("package.accept_keywords", "pkg/a amd64 ~x86")])?;

        let nodes = load_accept_keywords_configs(dir)?;
        assert_eq!(
            vec![ConfigNode {
                sources: vec![dir.join("package.accept_keywords")],
                value: ConfigNodeValue::AcceptKeywords(vec![AcceptKeywordsUpdate {
                    atom: PackageAtom::from_str("pkg/a").unwrap(),
                    accept_keywords: "amd64 ~x86".to_owned(),
                }]),
            },],
            nodes
        );
        Ok(())
    }

    #[test]
    fn test_load_accept_keywords_configs_load_dir() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                ("package.accept_keywords/aaa/bbb", "pkg/b amd64 ~x86"),
                ("package.accept_keywords/ccc", "pkg/c"),
            ],
        )?;

        let nodes = load_accept_keywords_configs(dir)?;
        assert_eq!(
            vec![
                ConfigNode {
                    sources: vec![dir.join("package.accept_keywords/aaa/bbb")],
                    value: ConfigNodeValue::AcceptKeywords(vec![AcceptKeywordsUpdate {
                        atom: PackageAtom::from_str("pkg/b").unwrap(),
                        accept_keywords: "amd64 ~x86".to_owned(),
                    }]),
                },
                ConfigNode {
                    sources: vec![dir.join("package.accept_keywords/ccc")],
                    value: ConfigNodeValue::AcceptKeywords(vec![AcceptKeywordsUpdate {
                        atom: PackageAtom::from_str("pkg/c").unwrap(),
                        accept_keywords: "".to_owned(),
                    }]),
                },
            ],
            nodes
        );
        Ok(())
    }
}
