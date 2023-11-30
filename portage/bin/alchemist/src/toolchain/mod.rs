// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use nom::Offset;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

use crate::repository::Repository;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainOptions {
    pub default: Option<bool>,
    pub sdk: Option<bool>,
    pub crossdev: Option<String>,
}

/// Represents a cross compiler tool chain
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toolchain {
    pub name: String,
    pub arch: String,
    pub options: Option<ToolchainOptions>,
}

impl FromStr for Toolchain {
    type Err = anyhow::Error;

    /// Parses a toolchain triple.
    ///
    /// i.e., x86_64-cros-linux-gnu
    ///
    /// It's also possible to attach JSON config settings.
    /// i.e., x86_64-cros-linux-gnu {"default": false}
    fn from_str(line: &str) -> Result<Self> {
        let mut whitespace = line.split_whitespace();

        let triple = match whitespace.next() {
            Some(triple) => triple,
            None => bail!("Tripplet is missing"),
        };

        // See https://wiki.osdev.org/Target_Triplet
        let mut parts = triple.split('-');
        let arch = match parts.next() {
            Some(arch) => arch,
            None => bail!("Tripplet is missing arch"),
        };

        // TODO: Do we need to parse the rest of the triple?

        // Ideally we would use something like splitn but it doesn't support
        // multiple whitespce characters in a row.
        let json = match whitespace.next() {
            Some(open_brace) => {
                let skip = line.offset(open_brace);
                Some(&line[skip..line.len()])
            }
            None => None,
        };

        let options: Option<ToolchainOptions> = match json {
            Some(json) => Some(serde_json::from_str(json)?),
            None => None,
        };

        Ok(Toolchain {
            name: triple.to_owned(),
            arch: arch.to_owned(),
            options,
        })
    }
}

impl Toolchain {
    pub fn can_be_default(&self) -> bool {
        match &self.options {
            Some(options) => options.default.unwrap_or(true),
            None => true,
        }
    }

    pub fn portage_arch(&self) -> Result<&'static str> {
        // Extracted using:
        // $ crossdev --show-target-cfg x86_64-cros-linux-gnu
        match self.arch.as_str() {
            "aarch64" => Ok("arm64"),
            "arm" => Ok("arm"),
            "armv7a" => Ok("arm"),
            "armv7m" => Ok("arm"),
            "i686" => Ok("x86"),
            "x86_64" => Ok("amd64"),
            _ => bail!("Unknown arch {}", &self.arch),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolchainConfig {
    pub toolchains: Vec<Toolchain>,
    pub default_index: Option<usize>,
}

/// Contains the toolchain configuration for a repository set.
impl ToolchainConfig {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ToolchainConfig {
            toolchains: Vec::new(),
            default_index: None,
        }
    }

    pub fn primary(&self) -> Option<&Toolchain> {
        match self.default_index {
            Some(index) => self.toolchains.get(index),
            None => None,
        }
    }

    fn get(&self, name: &str) -> Option<&Toolchain> {
        self.toolchains.iter().find(|t| t.name == name)
    }

    /// Loads the toolchain.conf that specifies the toolchains that are needed
    /// to build the board.
    ///
    /// See https://www.chromium.org/chromium-os/how-tos-and-troubleshooting/chromiumos-board-porting-guide/#toolchainconf
    /// for an example.
    ///
    /// Toolchain's also support loading JSON metadata:
    /// i.e., x86_64-cros-linux-gnu {"default": false}
    pub fn load(&mut self, path: &Path) -> Result<()> {
        let clean = Regex::new(r"\s*#.*$")?;

        let reader = BufReader::new(File::open(path)?);

        for result in reader.lines() {
            let line = result?;
            let line = clean.replace(&line, "");
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let toolchain = Toolchain::from_str(line)?;

            // For some reason the default toolchain is chosen by the most
            // specific overlay. i.e., overlay-$BOARD. This means that the board
            // overlays have to redeclare the "primary" toolchain that the
            // chipset or baseboard already specified. When we parse the
            // baseboard or chipset overlays we discard their duplicate
            // toolchain declarations since the board overlay already declared
            // it.
            if let Some(existing) = self.get(&toolchain.name) {
                if toolchain.options != existing.options {
                    // TODO: AFAIK there is no real-world usage of toolchain
                    // overrides. If we do have a need, we can implement settings
                    // overrides.
                    bail!(
                        "Duplicate toolchain ({}) declaration found in {}: {:#?}",
                        toolchain.name,
                        path.display(),
                        self.toolchains
                    );
                }
            } else {
                let can_be_default = toolchain.can_be_default();

                self.toolchains.push(toolchain);

                if self.default_index.is_none() && can_be_default {
                    self.default_index = Some(self.toolchains.len() - 1);
                }
            }
        }

        Ok(())
    }
}

pub fn load_toolchains(repos: &[&Repository]) -> Result<ToolchainConfig> {
    return load_toolchains_from_paths(
        repos
            .iter()
            .rev() // The primary toolchain is defined by the leaf overlay
            .map(|repo| repo.base_dir().join("toolchain.conf"))
            .collect(),
    );
}

// Helper to make unit testing possible
fn load_toolchains_from_paths<P: AsRef<Path>>(paths: Vec<P>) -> Result<ToolchainConfig> {
    let mut config = ToolchainConfig::new();

    for path in paths {
        if !path.as_ref().try_exists()? {
            continue;
        }

        config.load(path.as_ref())?;
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use crate::testutils::write_files;

    use super::*;

    #[test]
    fn test_toolchain() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [(
                "a/toolchain.conf",
                r#"# This is the main toolchain.
x86_64-cros-linux-gnu # With a comment at the end

# This is needed to build the AP/BIOS firmware.
i686-cros-linux-gnu

# This is needed to build the EC firmware
arm-none-eabi
"#,
            )],
        )?;

        let config = load_toolchains_from_paths(vec![
            dir.join("a/toolchain.conf"),
            dir.join("b/toolchain.conf"),
        ])?;

        assert_eq!(
            config,
            ToolchainConfig {
                default_index: Some(0),
                toolchains: vec![
                    Toolchain::from_str("x86_64-cros-linux-gnu")?,
                    Toolchain::from_str("i686-cros-linux-gnu")?,
                    Toolchain::from_str("arm-none-eabi")?,
                ]
            }
        );

        Ok(())
    }

    #[test]
    fn test_multiple_toolchain() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                (
                    "a/toolchain.conf",
                    r#"x86_64-cros-linux-gnu {"default": false}"#,
                ),
                ("b/toolchain.conf", "arm-none-eabi"),
            ],
        )?;

        let config = load_toolchains_from_paths(vec![
            dir.join("a/toolchain.conf"),
            dir.join("b/toolchain.conf"),
        ])?;

        assert_eq!(
            config,
            ToolchainConfig {
                default_index: Some(1),
                toolchains: vec![
                    Toolchain::from_str(r#"x86_64-cros-linux-gnu {"default": false}"#)?,
                    Toolchain::from_str("arm-none-eabi")?,
                ],
            }
        );

        assert_eq!(config.primary().unwrap(), &config.toolchains[1]);

        Ok(())
    }

    #[test]
    fn test_duplicate_toolchain() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                ("a/toolchain.conf", "x86_64-cros-linux-gnu\narm-none-eabi"),
                ("b/toolchain.conf", "x86_64-cros-linux-gnu"),
            ],
        )?;

        let config = load_toolchains_from_paths(vec![
            dir.join("a/toolchain.conf"),
            dir.join("b/toolchain.conf"),
        ])?;

        assert_eq!(
            config,
            ToolchainConfig {
                default_index: Some(0),
                toolchains: vec![
                    Toolchain::from_str("x86_64-cros-linux-gnu")?,
                    Toolchain::from_str("arm-none-eabi")?,
                ],
            }
        );

        Ok(())
    }

    #[test]
    fn test_duplicate_toolchain_with_override() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let dir = dir.as_ref();

        write_files(
            dir,
            [
                ("a/toolchain.conf", "x86_64-cros-linux-gnu"),
                (
                    "b/toolchain.conf",
                    r#"x86_64-cros-linux-gnu {"default": false}"#,
                ),
            ],
        )?;

        let config = load_toolchains_from_paths(vec![
            dir.join("a/toolchain.conf"),
            dir.join("b/toolchain.conf"),
        ]);

        assert!(config.is_err(), "Expected duplicate toolchains to fail");

        Ok(())
    }

    #[test]
    fn test_toolchain_json_whitespace() -> Result<()> {
        let toolchain = Toolchain::from_str(r#"x86_64-cros-linux-gnu   {"default": false}"#)?;

        assert_eq!(
            toolchain,
            Toolchain {
                name: "x86_64-cros-linux-gnu".to_owned(),
                arch: "x86_64".to_owned(),
                options: Some(ToolchainOptions {
                    default: Some(false),
                    sdk: None,
                    crossdev: None,
                })
            }
        );

        Ok(())
    }

    #[test]
    fn test_toolchain_simple() -> Result<()> {
        let toolchain = Toolchain::from_str("x86_64-cros-linux-gnu")?;

        assert_eq!(
            toolchain,
            Toolchain {
                name: "x86_64-cros-linux-gnu".to_owned(),
                arch: "x86_64".to_owned(),
                options: None,
            }
        );

        Ok(())
    }
}
