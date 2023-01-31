// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use itertools::Itertools;
use std::{path::Path, path::PathBuf};
use walkdir::WalkDir;

fn find_files(root: &Path, predicate: fn(&str) -> bool) -> Result<Vec<PathBuf>> {
    // Don't fail if root doesn't exist
    if let Err(e) = std::fs::metadata(root) {
        if e.kind() == std::io::ErrorKind::NotFound {
            return Ok(vec![]);
        }
        return Err(e.into());
    }

    let mut result = Vec::new();
    for entry in WalkDir::new(root) {
        let entry = entry?;
        let file_name = entry.path().file_name().unwrap().to_string_lossy();
        if predicate(&file_name) {
            result.push(entry.path().to_path_buf());
        }
    }
    Ok(result)
}

fn sort_contents(pkg_dir: &Path) -> Result<()> {
    for path in find_files(pkg_dir, |file_name| file_name == "CONTENTS")? {
        let contents = std::fs::read_to_string(&path)?
            .split("\n")
            .filter(|line| !line.is_empty())
            .sorted()
            .join("\n");
        std::fs::write(path, contents)?;
    }
    Ok(())
}

fn zero_counter(pkg_dir: &Path) -> Result<()> {
    for path in find_files(pkg_dir, |file_name| file_name == "COUNTER")? {
        std::fs::write(path, "0")?;
    }
    Ok(())
}

fn truncate_environment(pkg_dir: &Path) -> Result<()> {
    for path in find_files(pkg_dir, |file_name| file_name == "environment.bz2")? {
        std::fs::write(path, "")?;
    }
    Ok(())
}

fn clean_portage_database(root: &Path) -> Result<()> {
    // The portage database contains some non-hermetic install artifacts:
    // COUNTER: Since we are installing packages in parallel the COUNTER variable
    //          can change depending on when it was installed.
    // environment.bz2: The environment contains EPOCHTIME and SRANDOM from when the
    //                  package was installed. We could modify portage to omit these,
    //                  but I didn't think the binpkg-hermetic FEATURE should apply
    //                  to locally installed artifacts. So we just delete the file
    //                  for now.
    // CONTENTS: This file is sorted in the binpkg, but when portage installs the
    //           binpkg it recreates it in a non-hermetic way, so we manually sort
    //           it.
    // Deleting the files causes a "special" delete marker to be created by overlayfs
    // this isn't supported by bazel. So instead we just truncate the files.
    let pkg_dir = root.join("var/db/pkg");
    truncate_environment(&pkg_dir)?;
    zero_counter(&pkg_dir)?;
    sort_contents(&pkg_dir)?;
    Ok(())
}

pub fn clean_layer(board: Option<&str>, output_dir: &Path) -> Result<()> {
    let mut dirs = vec![
        PathBuf::from("mnt/host"),
        PathBuf::from("run"),
        PathBuf::from("stage"),
        PathBuf::from("tmp"),
        PathBuf::from("var/cache"),
        PathBuf::from("var/lib/portage/pkgs"),
        PathBuf::from("var/log"),
        PathBuf::from("var/tmp"),
    ];
    if let Some(b) = board {
        dirs.push(PathBuf::from("build").join(b).join("tmp"));
        dirs.push(PathBuf::from("build").join(b).join("var/cache"));
        dirs.push(PathBuf::from("build").join(b).join("packages"));
    }

    for dir in dirs {
        let path = output_dir.join(dir);
        fileutil::remove_dir_all_with_chmod(&path)?;
    }

    // So this is kind of annoying, since we monkey patch the portage .py files the
    // python interpreter will regenerate the bytecode cache. This bytecode file
    // has the timestamp of the source file embedded. Once we stop monkey patching
    // portage and get the changes bundled in the SDK we can delete the following.
    for file in find_files(
        &output_dir.join("usr/lib64/python3.6/site-packages"),
        |file_name| file_name.ends_with(".pyc"),
    )? {
        fileutil::remove_file_with_chmod(&file)?;
    }

    clean_portage_database(output_dir)?;
    if let Some(b) = board {
        clean_portage_database(&output_dir.join("build").join(b))?;
    }
    Ok(())
}
