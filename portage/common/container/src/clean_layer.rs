// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use fileutil::with_permissions;
use itertools::Itertools;
use libc::ENOTEMPTY;
use std::{io::ErrorKind, path::Path, path::PathBuf};
use tracing::instrument;
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

#[instrument]
fn sort_contents(pkg_dir: &Path) -> Result<()> {
    for path in find_files(pkg_dir, |file_name| file_name == "CONTENTS")? {
        let contents = std::fs::read_to_string(&path)?
            .split('\n')
            .filter(|line| !line.is_empty())
            .sorted()
            .interleave_shortest(std::iter::repeat("\n"))
            .join("");
        with_permissions(&path, 0o744, || {
            std::fs::write(&path, contents)
                .with_context(|| format!("Sorting CONTENTS for: {path:?}"))
        })?;
    }
    Ok(())
}

#[instrument]
fn zero_counter(pkg_dir: &Path) -> Result<()> {
    for path in find_files(pkg_dir, |file_name| file_name == "COUNTER")? {
        with_permissions(&path, 0o744, || {
            std::fs::write(&path, "0").with_context(|| format!("Clearing COUNTER for: {path:?}"))
        })?;
    }
    Ok(())
}

#[instrument]
fn truncate_environment(pkg_dir: &Path) -> Result<()> {
    for path in find_files(pkg_dir, |file_name| file_name == "environment.bz2")? {
        with_permissions(&path, 0o744, || {
            std::fs::write(&path, "")
                .with_context(|| format!("Zeroing environment.bz2 for: {path:?}"))
        })?;
    }
    Ok(())
}

#[instrument]
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

fn clean_root(root_dir: &Path) -> Result<()> {
    // So this is kind of annoying, since we monkey patch the portage .py files the
    // python interpreter will regenerate the bytecode cache. This bytecode file
    // has the timestamp of the source file embedded. Once we stop monkey patching
    // portage and get the changes bundled in the SDK we can delete the following.
    for file in find_files(
        &root_dir.join("usr/lib64/python3.6/site-packages"),
        |file_name| file_name.ends_with(".pyc"),
    )? {
        fileutil::remove_file_with_chmod(&file)?;
    }

    for subdir in [
        "mnt/host",
        "packages",
        "run",
        "stage",
        "tmp",
        "var/cache",
        "var/lib/portage/pkgs",
        "var/log",
        "var/tmp",
    ] {
        let target_dir = root_dir.join(subdir);
        fileutil::remove_dir_all_with_chmod(&target_dir)?;

        // Remove anscestors if they're empty.
        for dir in target_dir
            .ancestors()
            .skip(1)
            .take_while(|dir| *dir != root_dir)
        {
            match std::fs::remove_dir(dir) {
                Err(e) if e.kind() == ErrorKind::NotFound => continue,
                Err(e) if e.raw_os_error() == Some(ENOTEMPTY) => break,
                other => other.with_context(|| format!("Failed to delete {}", dir.display()))?,
            }
        }
    }

    clean_portage_database(root_dir)?;

    Ok(())
}

#[instrument]
pub fn clean_layer(output_dir: &Path) -> Result<()> {
    clean_root(output_dir)?;
    let build_dir = output_dir.join("build");
    if build_dir.try_exists()? {
        for entry in std::fs::read_dir(build_dir)? {
            let entry = entry?;
            if entry.metadata()?.is_dir() {
                clean_root(&entry.path())?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use fileutil::SafeTempDir;

    use super::*;

    #[test]
    fn test_clean_layer_remove_dirs() -> Result<()> {
        let output_dir = SafeTempDir::new()?;
        let output_dir = output_dir.path();

        for subdir in [
            // These directories will be deleted.
            "build/foo/mnt/host",
            "build/foo/run",
            "build/foo/stage",
            "build/foo/tmp",
            "build/foo/usr/lib64/python3.6/site-packages",
            "build/foo/var/cache",
            "build/foo/var/lib/portage/pkgs",
            "build/foo/var/log",
            "build/foo/var/tmp",
            "mnt/host",
            "run",
            "stage",
            "tmp",
            "var/cache",
            "var/lib/portage/pkgs",
            "var/log",
            "var/tmp",
            // These directories will be kept.
            "build/foo/opt",
            "build/foo/sbin",
            "build/foo/usr/bin",
            "build/foo/var/mail",
            "opt",
            "sbin",
            "usr/bin",
            "var/lib/keep",
            "usr/lib64/python3.6/site-packages",
            "var/mail",
        ] {
            std::fs::create_dir_all(output_dir.join(subdir))?;
        }

        clean_layer(output_dir)?;

        let paths: Vec<PathBuf> = WalkDir::new(output_dir)
            .min_depth(1)
            .sort_by_file_name()
            .into_iter()
            .map(|entry| Ok(entry?.path().strip_prefix(output_dir)?.to_path_buf()))
            .collect::<Result<_>>()?;

        assert_eq!(
            paths,
            vec![
                PathBuf::from("build"),
                PathBuf::from("build/foo"),
                PathBuf::from("build/foo/opt"),
                PathBuf::from("build/foo/sbin"),
                PathBuf::from("build/foo/usr"),
                PathBuf::from("build/foo/usr/bin"),
                PathBuf::from("build/foo/usr/lib64"),
                PathBuf::from("build/foo/usr/lib64/python3.6"),
                PathBuf::from("build/foo/usr/lib64/python3.6/site-packages"),
                PathBuf::from("build/foo/var"),
                PathBuf::from("build/foo/var/mail"),
                PathBuf::from("opt"),
                PathBuf::from("sbin"),
                PathBuf::from("usr"),
                PathBuf::from("usr/bin"),
                PathBuf::from("usr/lib64"),
                PathBuf::from("usr/lib64/python3.6"),
                PathBuf::from("usr/lib64/python3.6/site-packages"),
                PathBuf::from("var"),
                PathBuf::from("var/lib"),
                PathBuf::from("var/lib/keep"),
                PathBuf::from("var/mail"),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_clean_layer_canonicalize_vdb() -> Result<()> {
        let output_dir = SafeTempDir::new()?;
        let output_dir = output_dir.path();

        let vdb_dir = output_dir.join("build/foo/var/db/pkg/sys-apps/bar-1.0");

        std::fs::create_dir_all(&vdb_dir)?;
        std::fs::write(
            vdb_dir.join("CONTENTS"),
            r#"dir usr
dir usr/bin
obj usr/bin/world d41d8cd98f00b204e9800998ecf8427e 1111
obj usr/bin/hello d41d8cd98f00b204e9800998ecf8427e 2222
dir bin
sym bin/world -> ../usr/bin/world 3333
sym bin/hello -> /usr/bin/hello 4444
"#,
        )?;
        std::fs::write(vdb_dir.join("COUNTER"), "12345")?;
        std::fs::write(vdb_dir.join("environment.bz2"), "fake environment")?;

        clean_layer(output_dir)?;

        let contents = std::fs::read_to_string(vdb_dir.join("CONTENTS"))?;
        assert_eq!(
            contents,
            r#"dir bin
dir usr
dir usr/bin
obj usr/bin/hello d41d8cd98f00b204e9800998ecf8427e 2222
obj usr/bin/world d41d8cd98f00b204e9800998ecf8427e 1111
sym bin/hello -> /usr/bin/hello 4444
sym bin/world -> ../usr/bin/world 3333
"#
        );
        let counter = std::fs::read_to_string(vdb_dir.join("COUNTER"))?;
        assert_eq!(counter, "0");
        let environment = std::fs::read_to_string(vdb_dir.join("environment.bz2"))?;
        assert_eq!(environment, "");

        Ok(())
    }
}
