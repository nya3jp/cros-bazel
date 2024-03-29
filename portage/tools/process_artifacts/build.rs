// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    io::Result,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let mut includes: Vec<PathBuf> = vec!["proto/third_party".into()];
    // Under Bazel, $WELL_KNOWN_TYPES_MARKER contains a path to the LICENSE file in the protobuf
    // repository. Use this path to locate well know type protos.
    if let Some(marker_path) = std::env::var_os("WELL_KNOWN_TYPES_MARKER") {
        includes.push(Path::new(&marker_path).parent().unwrap().join("src"));
    }
    prost_build::compile_protos(&["proto/third_party/spawn.proto"], &includes)?;
    Ok(())
}
