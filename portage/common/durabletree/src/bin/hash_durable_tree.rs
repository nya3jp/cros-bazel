// Copyright 2026 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use durabletree::DurableTree;
use std::path::Path;

/// Prints a deterministic SHA256 hash of a durable tree's content.
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        bail!("usage: {} <durable-tree-root>", args[0]);
    }
    println!("{}", DurableTree::content_hash(Path::new(&args[1]))?);
    Ok(())
}
