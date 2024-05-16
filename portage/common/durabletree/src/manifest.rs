// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Defines the format of a file entry in `manifest.json`.
#[derive(Debug, Serialize, Deserialize)]
pub enum FileEntry {
    #[serde(rename = "f")]
    Regular {
        #[serde(rename = "m")]
        mode: u32,
        #[serde(rename = "x", skip_serializing_if = "BTreeMap::is_empty", default)]
        user_xattrs: BTreeMap<String, Vec<u8>>,
    },
    #[serde(rename = "d")]
    Directory {
        #[serde(rename = "m")]
        mode: u32,
        #[serde(rename = "x", skip_serializing_if = "BTreeMap::is_empty", default)]
        user_xattrs: BTreeMap<String, Vec<u8>>,
    },
    #[serde(rename = "s")]
    Symlink {
        #[serde(rename = "t")]
        target: String,
    },
    #[serde(rename = "w")]
    Whiteout,
}

/// Defines the format of `manifest.json` in a durable tree.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DurableTreeManifest {
    pub files: BTreeMap<String, FileEntry>,
}
