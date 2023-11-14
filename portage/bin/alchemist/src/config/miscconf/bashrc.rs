// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use anyhow::{ensure, Context};
use itertools::Itertools;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::vec;

use crate::config::{ConfigNode, ConfigNodeValue, PackageBashrc};
use crate::dependency::package::PackageAtom;

pub fn load_bashrc(dir: &Path) -> Result<Vec<ConfigNode>> {
    let mut nodes: Vec<ConfigNode> = vec![];

    nodes.extend(load_profile_bashrc(dir)?);
    nodes.extend(load_package_bashrc(dir)?);

    Ok(nodes)
}

fn get_sources(source: PathBuf) -> Result<Vec<PathBuf>> {
    if !source.exists() {
        return Ok(Vec::new());
    }

    // Load child files and directories if it's a directory.
    if source.is_dir() {
        let mut sources = vec![];
        for entry in source.read_dir()? {
            let entry = entry?;
            let name = source.join(entry.file_name());

            if !entry.file_type()?.is_file() {
                bail!("'{}' is not a file", name.display())
            }

            sources.push(name);
        }

        sources.sort();

        Ok(sources)
    } else {
        Ok(vec![source])
    }
}

fn load_profile_bashrc(dir: &Path) -> Result<Vec<ConfigNode>> {
    let sources = get_sources(dir.join("profile.bashrc"))?;

    if sources.is_empty() {
        return Ok(vec![]);
    }

    Ok(vec![ConfigNode {
        value: ConfigNodeValue::ProfileBashrc(sources.to_vec()),
        sources,
    }])
}

fn parse_package_bashrc_line(bashrc_dir: &Path, line: &str) -> Result<PackageBashrc> {
    let line = line.trim();
    let mut parts = line.split_whitespace();

    let atom = parts
        .next()
        .context("Missing atom")?
        .parse::<PackageAtom>()?;

    let paths = parts
        .map(|relative_bashrc| bashrc_dir.join(relative_bashrc))
        .collect_vec();
    ensure!(!paths.is_empty(), "Missing path");

    Ok(PackageBashrc { atom, paths })
}

fn parse_package_bashrc(bashrc_dir: &Path, source: &Path) -> Result<Vec<PackageBashrc>> {
    let contents = read_to_string(source)?;

    contents
        .split('\n')
        .map(|line| line.trim())
        .enumerate()
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .map(|(lineno, line)| (lineno, parse_package_bashrc_line(bashrc_dir, line)))
        .map(|(lineno, result)| {
            result
                .and_then(|value| {
                    for path in &value.paths {
                        ensure!(path.try_exists()?, "{} does not exist", path.display());
                    }
                    Ok(value)
                })
                .with_context(|| {
                    format!(
                        "Failed to load {}: error at line {}",
                        source.display(),
                        lineno + 1
                    )
                })
        })
        .collect()
}

fn load_package_bashrc(dir: &Path) -> Result<Vec<ConfigNode>> {
    let sources = get_sources(dir.join("package.bashrc"))?;
    let bashrc_dir = dir.join("bashrc");

    let mut nodes = vec![];
    for source in sources {
        nodes.push(ConfigNode {
            value: ConfigNodeValue::PackageBashrcs(parse_package_bashrc(&bashrc_dir, &source)?),
            sources: vec![source],
        });
    }

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use crate::testutils::write_files;

    use super::*;

    #[test]
    fn test_load_profile_bashrc_single_file() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [(
                "profile.bashrc",
                r#"
                    #!/bin/bash
                    echo Hello World
                "#,
            )],
        )?;

        let nodes = load_profile_bashrc(dir)?;
        assert_eq!(
            vec![ConfigNode {
                sources: vec![dir.join("profile.bashrc")],
                value: ConfigNodeValue::ProfileBashrc(vec![dir.join("profile.bashrc")]),
            }],
            nodes
        );
        Ok(())
    }

    #[test]
    fn test_load_profile_bashrc_multiple_files() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir.join("profile.bashrc"),
            [
                (
                    "first",
                    r#"
                    #!/bin/bash
                    echo Hello
                "#,
                ),
                (
                    "second",
                    r#"
                    #!/bin/bash
                    echo World
                "#,
                ),
            ],
        )?;

        let nodes = load_profile_bashrc(dir)?;
        assert_eq!(
            vec![ConfigNode {
                sources: vec![
                    dir.join("profile.bashrc/first"),
                    dir.join("profile.bashrc/second"),
                ],
                value: ConfigNodeValue::ProfileBashrc(vec![
                    dir.join("profile.bashrc/first"),
                    dir.join("profile.bashrc/second"),
                ]),
            }],
            nodes
        );
        Ok(())
    }

    #[test]
    fn test_load_package_bashrc_single_file() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                ("bashrc/cross-arm-none-eabi/other.sh", ""),
                ("bashrc/cross-arm-none-eabi/binutils.sh", ""),
                ("bashrc/cross-arm-none-eabi/gcc.sh", ""),
                (
                    "package.bashrc",
                    r#"
# Hello world
cross-arm-none-eabi/binutils cross-arm-none-eabi/binutils.sh

cross-arm-none-eabi/gcc cross-arm-none-eabi/gcc.sh cross-arm-none-eabi/other.sh
                "#,
                ),
            ],
        )?;

        let nodes = load_package_bashrc(dir)?;
        assert_eq!(
            vec![ConfigNode {
                sources: vec![dir.join("package.bashrc")],
                value: ConfigNodeValue::PackageBashrcs(vec![
                    PackageBashrc {
                        atom: "cross-arm-none-eabi/binutils".parse()?,
                        paths: vec![dir.join("bashrc").join("cross-arm-none-eabi/binutils.sh")],
                    },
                    PackageBashrc {
                        atom: "cross-arm-none-eabi/gcc".parse()?,
                        paths: vec![
                            dir.join("bashrc").join("cross-arm-none-eabi/gcc.sh"),
                            dir.join("bashrc").join("cross-arm-none-eabi/other.sh"),
                        ],
                    },
                ]),
            }],
            nodes
        );
        Ok(())
    }
}
