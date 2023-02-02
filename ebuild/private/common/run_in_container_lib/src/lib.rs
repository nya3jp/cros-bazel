// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
use anyhow::Result;
use makechroot::{BindMount, OverlayInfo};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct RunInContainerConfig {
    /// Directory that will be used by the overlayfs mount.
    /// Output artifacts can be found in the 'diff' directory.
    pub staging_dir: PathBuf,

    /// Directory to use as the working directory while inside the namespace.
    pub chdir: PathBuf,

    /// Overlays to be mounted in the namespace. The earlier overlays are
    /// mounted as the higher layer, and the later overlays are mounted as the
    /// lower layer.
    pub overlays: Vec<OverlayInfo>,

    /// Bind-mounts to apply. Applies on top of overlays, and can mount
    /// individual files as well as directories.
    pub bind_mounts: Vec<BindMount>,

    /// If true, the contents of the host machine are mounted at /host.
    pub keep_host_mount: bool,
}

impl RunInContainerConfig {
    pub fn deserialize_from(path: &Path) -> Result<Self> {
        Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
    }

    pub fn serialize_to(&self, path: &Path) -> Result<()> {
        serde_json::to_writer(File::create(path)?, self)?;
        Ok(())
    }
}
