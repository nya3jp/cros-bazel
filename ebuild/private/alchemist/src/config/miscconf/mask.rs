// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use itertools::Itertools;
use std::{fs::read_to_string, path::Path};

use crate::{
    config::{ConfigNode, ConfigNodeValue, PackageMaskKind, PackageMaskUpdate},
    dependency::package::PackageAtomDependency,
};

fn load_package_config(source: &Path, kind: PackageMaskKind) -> Result<Vec<ConfigNode>> {
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
                load_package_config(&new_source, kind)
                    .with_context(|| format!("Loading {}", new_source.to_string_lossy()))?,
            );
        }
        return Ok(nodes);
    }

    let contents = read_to_string(source)?;

    let mut updates = Vec::<PackageMaskUpdate>::new();

    for line in contents
        .split("\n")
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("#"))
    {
        let atom = line.trim().parse::<PackageAtomDependency>()?;
        updates.push(PackageMaskUpdate { kind, atom })
    }

    Ok(vec![ConfigNode {
        source: source.to_owned(),
        value: ConfigNodeValue::PackageMasks(updates),
    }])
}

pub fn load_package_configs(dir: &Path) -> Result<Vec<ConfigNode>> {
    Ok([
        load_package_config(&dir.join("package.mask"), PackageMaskKind::Mask)?,
        load_package_config(&dir.join("package.unmask"), PackageMaskKind::Unmask)?,
    ]
    .into_iter()
    .concat())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::testutils::write_files;

    use super::*;

    #[test]
    fn test_load_package_configs() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                ("package.mask", "pkg/a\n=pkg/b-1.0.0"),
                ("package.unmask", "pkg/c\n=pkg/d-1.0.0"),
            ],
        )?;

        let nodes = load_package_configs(dir)?;
        assert_eq!(
            vec![
                ConfigNode {
                    source: dir.join("package.mask"),
                    value: ConfigNodeValue::PackageMasks(vec![
                        PackageMaskUpdate {
                            kind: PackageMaskKind::Mask,
                            atom: PackageAtomDependency::new_simple("pkg/a"),
                        },
                        PackageMaskUpdate {
                            kind: PackageMaskKind::Mask,
                            atom: PackageAtomDependency::from_str("=pkg/b-1.0.0").unwrap(),
                        },
                    ]),
                },
                ConfigNode {
                    source: dir.join("package.unmask"),
                    value: ConfigNodeValue::PackageMasks(vec![
                        PackageMaskUpdate {
                            kind: PackageMaskKind::Unmask,
                            atom: PackageAtomDependency::new_simple("pkg/c"),
                        },
                        PackageMaskUpdate {
                            kind: PackageMaskKind::Unmask,
                            atom: PackageAtomDependency::from_str("=pkg/d-1.0.0").unwrap(),
                        },
                    ]),
                },
            ],
            nodes
        );
        Ok(())
    }

    #[test]
    fn test_load_package_configs_directory() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                ("package.mask/a.conf", "pkg/a"),
                ("package.mask/b.conf", "pkg/b"),
                ("package.unmask/c.conf", "pkg/c"),
                ("package.unmask/d.conf", "pkg/d"),
            ],
        )?;

        let nodes = load_package_configs(dir)?;
        assert_eq!(
            vec![
                ConfigNode {
                    source: dir.join("package.mask/a.conf"),
                    value: ConfigNodeValue::PackageMasks(vec![PackageMaskUpdate {
                        kind: PackageMaskKind::Mask,
                        atom: PackageAtomDependency::from_str("pkg/a").unwrap(),
                    }]),
                },
                ConfigNode {
                    source: dir.join("package.mask/b.conf"),
                    value: ConfigNodeValue::PackageMasks(vec![PackageMaskUpdate {
                        kind: PackageMaskKind::Mask,
                        atom: PackageAtomDependency::from_str("pkg/b").unwrap(),
                    }]),
                },
                ConfigNode {
                    source: dir.join("package.unmask/c.conf"),
                    value: ConfigNodeValue::PackageMasks(vec![PackageMaskUpdate {
                        kind: PackageMaskKind::Unmask,
                        atom: PackageAtomDependency::from_str("pkg/c").unwrap(),
                    }]),
                },
                ConfigNode {
                    source: dir.join("package.unmask/d.conf"),
                    value: ConfigNodeValue::PackageMasks(vec![PackageMaskUpdate {
                        kind: PackageMaskKind::Unmask,
                        atom: PackageAtomDependency::from_str("pkg/d").unwrap(),
                    }]),
                },
            ],
            nodes
        );
        Ok(())
    }
}
