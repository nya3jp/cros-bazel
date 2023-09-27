// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

/// Useful for serde.
fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    v == &T::default()
}

/// A unique *stable* identifier for a package.
/// package version is unsuitable here because we don't want uprevs to modify the package id.
#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Clone, Debug)]
pub struct PackageUid {
    pub name: String,
    pub slot: String,
}

// This type isn't useful, but just allows serde to skip if the serialization is empty.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    #[serde(flatten)]
    #[serde(default, skip_serializing_if = "is_default")]
    pub file_type: FileType,
}

// A pathbuf would be better, but this allows serde to serialize it.
// https://github.com/serde-rs/serde/issues/1307
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SymlinkMetadata {
    pub target: PathBuf,
}

// When changing this, also change bazel/portage/build_defs/extract_package_from_manifest/files.bzl
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum FileType {
    #[default]
    Unknown,
    Symlink(SymlinkMetadata),
    HeaderFile,
    SharedLibrary,
    ElfBinary,
}

/// A package, including both analysis-phase metadata accessible to bazel, and runtime metadata
/// like package contents accessible to the actions.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Package {
    #[serde(flatten)]
    pub uid: PackageUid,
    pub content: BTreeMap<PathBuf, FileMetadata>,
}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}
