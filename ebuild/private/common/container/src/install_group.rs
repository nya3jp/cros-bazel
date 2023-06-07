// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Error, Result};
use binarypackage::BinaryPackage;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::BindMount;

#[derive(Debug, Clone)]
pub struct InstallGroup {
    packages: Vec<PathBuf>,
}

impl FromStr for InstallGroup {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self {
            packages: s.split(':').map(PathBuf::from).collect(),
        })
    }
}

impl InstallGroup {
    fn get_config(&self, dir: &Path) -> Result<(Vec<BindMount>, Vec<String>)> {
        let mut bind_mounts: Vec<BindMount> = Vec::new();
        let mut atoms: Vec<String> = Vec::new();
        for package in &self.packages {
            let bp = BinaryPackage::open(package)?;
            let category_pf = bp.category_pf();
            bind_mounts.push(BindMount {
                source: package.into(),
                mount_path: dir.join(format!("{category_pf}.tbz2")),
                rw: false,
            });
            atoms.push(format!("={category_pf}"));
        }
        Ok((bind_mounts, atoms))
    }

    pub fn get_mounts_and_env<P: AsRef<Path>>(
        install_groups: &[InstallGroup],
        dir: P,
    ) -> Result<(Vec<BindMount>, BTreeMap<OsString, OsString>)> {
        let mut bind_mounts: Vec<BindMount> = Vec::new();
        let mut env: BTreeMap<OsString, OsString> = BTreeMap::new();
        for (i, install_group) in install_groups.iter().enumerate() {
            let (mut group_mounts, atoms) = install_group.get_config(dir.as_ref())?;
            bind_mounts.append(&mut group_mounts);
            env.insert(
                format!("INSTALL_ATOMS_TARGET_{i}").into(),
                atoms.join(" ").into(),
            );
        }
        Ok((bind_mounts, env))
    }
}
