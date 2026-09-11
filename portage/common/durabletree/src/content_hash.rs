// Copyright 2026 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{fs::File, io::BufReader, path::Path};

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use crate::{
    consts::{MANIFEST_FILE_NAME, MARKER_FILE_NAME, RAW_DIR_NAME},
    manifest::{DurableTreeManifest, FileEntry},
};

fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).with_context(|| format!("read {}", path.display()))?;
    Ok(hex::encode(hasher.finalize()))
}

/// See [`DurableTree::content_hash`](crate::DurableTree::content_hash).
pub fn content_hash_impl(root_dir: &Path) -> Result<String> {
    if !root_dir.join(MARKER_FILE_NAME).try_exists()? {
        bail!("{} is not a durable tree", root_dir.display());
    }

    let manifest: DurableTreeManifest = serde_json::from_reader(BufReader::new(File::open(
        root_dir.join(MANIFEST_FILE_NAME),
    )?))?;

    // Fold "<sha256>  <path>" lines in the manifest's deterministic order.
    // manifest.json covers all metadata; content of files whose mode has no
    // read bits is skipped, since restoration makes them unreadable.
    let mut folded = Sha256::new();
    let digest = hash_file(&root_dir.join(MANIFEST_FILE_NAME))?;
    folded.update(format!("{}  ./{}\n", digest, MANIFEST_FILE_NAME));

    let raw_dir = root_dir.join(RAW_DIR_NAME);
    for (relative_path, entry) in &manifest.files {
        match entry {
            FileEntry::Regular { mode, .. } if mode & 0o444 != 0 => {
                let digest = hash_file(&raw_dir.join(relative_path))?;
                folded.update(format!(
                    "{}  ./{}/{}\n",
                    digest, RAW_DIR_NAME, relative_path
                ));
            }
            _ => {}
        }
    }

    Ok(hex::encode(folded.finalize()))
}
