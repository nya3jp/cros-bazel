// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Defines the format of a file entry in `manifest.json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileManifest {
    #[serde(rename = "m")]
    pub mode: u32,
    #[serde(rename = "x", skip_serializing_if = "BTreeMap::is_empty", default)]
    pub user_xattrs: BTreeMap<String, Vec<u8>>,
}

/// Defines the format of `manifest.json` in a durable tree.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DurableTreeManifest {
    pub files: BTreeMap<String, FileManifest>,
}
