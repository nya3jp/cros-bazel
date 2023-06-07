// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{path::PathBuf, str::FromStr};

use anyhow::{ensure, Result};
use run_in_container_lib::BindMountConfig;
use strum_macros::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, EnumString, strum_macros::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum LoginMode {
    #[strum(serialize = "")]
    Never,
    Before,
    After,
    AfterFail,
}

#[derive(Clone, Debug)]
pub struct BindMount {
    pub mount_path: PathBuf,
    pub source: PathBuf,
    pub rw: bool,
}

impl FromStr for BindMount {
    type Err = anyhow::Error;

    fn from_str(spec: &str) -> Result<Self> {
        let v: Vec<_> = spec.split('=').collect();
        ensure!(v.len() == 2, "Invalid bind-mount spec: {:?}", spec);
        Ok(Self {
            mount_path: v[0].into(),
            source: v[1].into(),
            rw: false,
        })
    }
}

impl BindMount {
    pub fn into_config(self) -> BindMountConfig {
        BindMountConfig {
            mount_path: self.mount_path,
            source: self.source,
            rw: self.rw,
        }
    }
}
