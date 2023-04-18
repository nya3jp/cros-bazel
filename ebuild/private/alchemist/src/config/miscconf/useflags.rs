// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use std::{fs::read_to_string, path::Path};

use crate::{
    config::{ConfigNode, ConfigNodeValue, UseUpdate, UseUpdateFilter, UseUpdateKind},
    dependency::package::PackageAtom,
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
                    .with_context(|| format!("Failed to load {}", new_source.to_string_lossy()))?,
            );
        }
        return Ok(nodes);
    }

    let contents = read_to_string(source)?;

    let mut updates = Vec::<UseUpdate>::new();

    for tokens in contents
        .split('\n')
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
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
                    .with_context(|| format!("Failed to load {}", source.display()))?,
            );
        }
        return Ok(nodes);
    }

    let contents = read_to_string(source)?;

    let mut updates = Vec::<UseUpdate>::new();

    for (lineno, line) in contents
        .split('\n')
        .map(|line| line.trim())
        .enumerate()
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
    {
        let (raw_atom, tokens) = line
            .trim()
            .split_once(|c: char| c.is_ascii_whitespace())
            .unwrap_or((line, ""));
        let atom = raw_atom
            .parse::<PackageAtom>()
            .with_context(|| format!("Failed to load {}: line {}", source.display(), lineno + 1))?;
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
    .concat())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::testutils::write_files;

    use super::*;

    #[test]
    fn test_load_use_configs() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                ("package.use", "pkg/a foo -bar baz"),
                ("use.mask", "foo -bar baz"),
                ("use.stable.mask", "foo -bar baz"),
                ("package.use.mask", "pkg/b foo -bar baz"),
                ("package.use.stable.mask", "pkg/c foo -bar baz"),
                ("use.force", "foo -bar baz"),
                ("use.stable.force", "foo -bar baz"),
                ("package.use.force", "pkg/d foo -bar baz"),
                ("package.use.stable.force", "pkg/e foo -bar baz"),
            ],
        )?;

        let nodes = load_use_configs(dir)?;
        assert_eq!(
            vec![
                ConfigNode {
                    source: dir.join("package.use"),
                    value: ConfigNodeValue::Uses(vec![UseUpdate {
                        kind: UseUpdateKind::Set,
                        filter: UseUpdateFilter {
                            atom: Some(PackageAtom::from_str("pkg/a").unwrap()),
                            stable_only: false,
                        },
                        use_tokens: "foo -bar baz".to_owned(),
                    }]),
                },
                ConfigNode {
                    source: dir.join("use.mask"),
                    value: ConfigNodeValue::Uses(vec![UseUpdate {
                        kind: UseUpdateKind::Mask,
                        filter: UseUpdateFilter {
                            atom: None,
                            stable_only: false,
                        },
                        use_tokens: "foo -bar baz".to_owned(),
                    }]),
                },
                ConfigNode {
                    source: dir.join("use.stable.mask"),
                    value: ConfigNodeValue::Uses(vec![UseUpdate {
                        kind: UseUpdateKind::Mask,
                        filter: UseUpdateFilter {
                            atom: None,
                            stable_only: true,
                        },
                        use_tokens: "foo -bar baz".to_owned(),
                    }]),
                },
                ConfigNode {
                    source: dir.join("package.use.mask"),
                    value: ConfigNodeValue::Uses(vec![UseUpdate {
                        kind: UseUpdateKind::Mask,
                        filter: UseUpdateFilter {
                            atom: Some(PackageAtom::from_str("pkg/b").unwrap()),
                            stable_only: false,
                        },
                        use_tokens: "foo -bar baz".to_owned(),
                    }]),
                },
                ConfigNode {
                    source: dir.join("package.use.stable.mask"),
                    value: ConfigNodeValue::Uses(vec![UseUpdate {
                        kind: UseUpdateKind::Mask,
                        filter: UseUpdateFilter {
                            atom: Some(PackageAtom::from_str("pkg/c").unwrap()),
                            stable_only: true,
                        },
                        use_tokens: "foo -bar baz".to_owned(),
                    }]),
                },
                ConfigNode {
                    source: dir.join("use.force"),
                    value: ConfigNodeValue::Uses(vec![UseUpdate {
                        kind: UseUpdateKind::Force,
                        filter: UseUpdateFilter {
                            atom: None,
                            stable_only: false,
                        },
                        use_tokens: "foo -bar baz".to_owned(),
                    }]),
                },
                ConfigNode {
                    source: dir.join("use.stable.force"),
                    value: ConfigNodeValue::Uses(vec![UseUpdate {
                        kind: UseUpdateKind::Force,
                        filter: UseUpdateFilter {
                            atom: None,
                            stable_only: true,
                        },
                        use_tokens: "foo -bar baz".to_owned(),
                    }]),
                },
                ConfigNode {
                    source: dir.join("package.use.force"),
                    value: ConfigNodeValue::Uses(vec![UseUpdate {
                        kind: UseUpdateKind::Force,
                        filter: UseUpdateFilter {
                            atom: Some(PackageAtom::from_str("pkg/d").unwrap()),
                            stable_only: false,
                        },
                        use_tokens: "foo -bar baz".to_owned(),
                    }]),
                },
                ConfigNode {
                    source: dir.join("package.use.stable.force"),
                    value: ConfigNodeValue::Uses(vec![UseUpdate {
                        kind: UseUpdateKind::Force,
                        filter: UseUpdateFilter {
                            atom: Some(PackageAtom::from_str("pkg/e").unwrap()),
                            stable_only: true,
                        },
                        use_tokens: "foo -bar baz".to_owned(),
                    }]),
                },
            ],
            nodes
        );
        Ok(())
    }
}
