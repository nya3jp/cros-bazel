// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use std::os::unix::fs::symlink;
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use crate::path::join_absolute;

pub enum FileOps {
    Symlink { target: PathBuf, source: PathBuf },
    PlainFile { path: PathBuf, content: String },
    Mkdir { path: PathBuf },
}

/// Helps to make init easier to read.
impl FileOps {
    pub fn symlink(target: impl AsRef<Path>, source: impl AsRef<Path>) -> Self {
        FileOps::Symlink {
            target: target.as_ref().to_owned(),
            source: source.as_ref().to_owned(),
        }
    }

    pub fn plainfile(path: impl AsRef<Path>, content: impl AsRef<str>) -> Self {
        FileOps::PlainFile {
            path: path.as_ref().to_owned(),
            content: content.as_ref().to_owned(),
        }
    }

    pub fn mkdir(path: impl AsRef<Path>) -> Self {
        FileOps::Mkdir {
            path: path.as_ref().to_owned(),
        }
    }
}

fn make_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent).with_context(|| format!("mkdir -p {}", parent.display()))?;
    }
    Ok(())
}

pub fn execute_file_ops(ops: &[FileOps], root: &Path) -> Result<()> {
    for op in ops {
        match op {
            FileOps::Symlink { target, source } => {
                let path = join_absolute(root, target)?;
                make_parent_dir(&path)?;

                symlink(source, &path)
                    .with_context(|| format!("ln -s {} {}", source.display(), path.display()))?
            }
            FileOps::PlainFile { path, content } => {
                let path = join_absolute(root, path)?;
                make_parent_dir(&path)?;

                std::fs::write(&path, content)
                    .with_context(|| format!("file {}", path.display()))?
            }
            FileOps::Mkdir { path } => {
                let path = join_absolute(root, path)?;
                create_dir_all(&path).with_context(|| format!("mkdir -p {}", path.display()))?;
            }
        }
    }

    Ok(())
}
