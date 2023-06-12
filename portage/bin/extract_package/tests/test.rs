// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{path::Path, process::Command};

use anyhow::{ensure, Result};
use durabletree::DurableTree;
use fileutil::SafeTempDir;
use runfiles::Runfiles;
use walkdir::WalkDir;

// Run unit tests in a mount namespace to use durable trees.
#[used]
#[link_section = ".init_array"]
static _CTOR: extern "C" fn() = ::testutil::ctor_enter_mount_namespace;

fn flatten_layers(layer_dirs: &[&Path], out_dir: &Path) -> Result<()> {
    for layer_dir in layer_dirs {
        let status = Command::new("/bin/cp")
            .args(["-a", "--"])
            .arg(layer_dir.join("."))
            .arg(out_dir)
            .status()?;
        ensure!(status.success(), "cp failed: {:?}", status);
    }
    Ok(())
}

fn run_extract_package(image_prefix: &str, vdb_prefix: &str, host: bool) -> Result<SafeTempDir> {
    let raw_out_dir = SafeTempDir::new()?;

    let runfiles = Runfiles::create()?;
    let program_path = runfiles.rlocation("cros/bazel/portage/bin/extract_package/extract_package");
    let binary_package_path = runfiles
        .rlocation("cros/bazel/portage/bin/extract_package/testdata/extract-test-1.2.3.tbz2");
    let status = Command::new(program_path)
        .arg("--input-binary-package")
        .arg(&binary_package_path)
        .arg("--output-directory")
        .arg(raw_out_dir.path())
        .arg("--image-prefix")
        .arg(image_prefix)
        .arg("--vdb-prefix")
        .arg(vdb_prefix)
        .args(if host { &["--host"][..] } else { &[][..] })
        .status()?;
    ensure!(status.success(), "extract_package failed: {:?}", status);

    // Expand the durable tree.
    DurableTree::cool_down_for_testing(raw_out_dir.path())?;
    let tree = DurableTree::expand(raw_out_dir.path())?;

    let out_dir = SafeTempDir::new()?;
    flatten_layers(&tree.layers(), out_dir.path())?;

    Ok(out_dir)
}

fn list_files_under(dir: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in WalkDir::new(dir).sort_by_file_name() {
        let entry = entry?;
        if entry.file_type().is_file() {
            let name = entry.path().strip_prefix(dir).unwrap();
            names.push(name.to_string_lossy().to_string());
        }
    }
    Ok(names)
}

#[test]
fn installed_image_for_target() -> Result<()> {
    let out_dir = run_extract_package("build/foo", "build/foo", false)?;
    assert_eq!(
        list_files_under(out_dir.path())?,
        vec![
            "build/foo/bin/helloworld",
            "build/foo/lib/helloworld",
            "build/foo/usr/bin/helloworld",
            "build/foo/usr/lib/helloworld",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/BUILD_TIME",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CATEGORY",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CBUILD",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CC",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CFLAGS",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CHOST",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CONTENTS",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/COUNTER",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CXX",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CXXFLAGS",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/DEFINED_PHASES",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/EAPI",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/FEATURES",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/IUSE",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/IUSE_EFFECTIVE",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/KEYWORDS",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/LDFLAGS",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/PF",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/PKG_INSTALL_MASK",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/SIZE",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/SLOT",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/USE",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/environment.bz2",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/extract-test-1.2.3.ebuild",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/license.yaml",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/repository"
        ]
    );
    Ok(())
}

#[test]
fn installed_image_for_host() -> Result<()> {
    let out_dir = run_extract_package("", "", true)?;
    assert_eq!(
        list_files_under(out_dir.path())?,
        vec![
            "bin/helloworld",
            "lib64/helloworld", // rewrote lib->lib64
            "usr/bin/helloworld",
            "usr/lib64/helloworld", // rewrote lib->lib64
            "var/db/pkg/virtual/extract-test-1.2.3/BUILD_TIME",
            "var/db/pkg/virtual/extract-test-1.2.3/CATEGORY",
            "var/db/pkg/virtual/extract-test-1.2.3/CBUILD",
            "var/db/pkg/virtual/extract-test-1.2.3/CC",
            "var/db/pkg/virtual/extract-test-1.2.3/CFLAGS",
            "var/db/pkg/virtual/extract-test-1.2.3/CHOST",
            "var/db/pkg/virtual/extract-test-1.2.3/CONTENTS",
            "var/db/pkg/virtual/extract-test-1.2.3/COUNTER",
            "var/db/pkg/virtual/extract-test-1.2.3/CXX",
            "var/db/pkg/virtual/extract-test-1.2.3/CXXFLAGS",
            "var/db/pkg/virtual/extract-test-1.2.3/DEFINED_PHASES",
            "var/db/pkg/virtual/extract-test-1.2.3/EAPI",
            "var/db/pkg/virtual/extract-test-1.2.3/FEATURES",
            "var/db/pkg/virtual/extract-test-1.2.3/IUSE",
            "var/db/pkg/virtual/extract-test-1.2.3/IUSE_EFFECTIVE",
            "var/db/pkg/virtual/extract-test-1.2.3/KEYWORDS",
            "var/db/pkg/virtual/extract-test-1.2.3/LDFLAGS",
            "var/db/pkg/virtual/extract-test-1.2.3/PF",
            "var/db/pkg/virtual/extract-test-1.2.3/PKG_INSTALL_MASK",
            "var/db/pkg/virtual/extract-test-1.2.3/SIZE",
            "var/db/pkg/virtual/extract-test-1.2.3/SLOT",
            "var/db/pkg/virtual/extract-test-1.2.3/USE",
            "var/db/pkg/virtual/extract-test-1.2.3/environment.bz2",
            "var/db/pkg/virtual/extract-test-1.2.3/extract-test-1.2.3.ebuild",
            "var/db/pkg/virtual/extract-test-1.2.3/license.yaml",
            "var/db/pkg/virtual/extract-test-1.2.3/repository"
        ]
    );
    Ok(())
}

#[test]
fn staged_image_for_target() -> Result<()> {
    let out_dir = run_extract_package(".image", "build/foo", false)?;
    assert_eq!(
        list_files_under(out_dir.path())?,
        vec![
            ".image/bin/helloworld",
            ".image/lib/helloworld",
            ".image/usr/bin/helloworld",
            ".image/usr/lib/helloworld",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/BUILD_TIME",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CATEGORY",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CBUILD",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CC",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CFLAGS",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CHOST",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CONTENTS",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/COUNTER",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CXX",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/CXXFLAGS",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/DEFINED_PHASES",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/EAPI",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/FEATURES",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/IUSE",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/IUSE_EFFECTIVE",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/KEYWORDS",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/LDFLAGS",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/PF",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/PKG_INSTALL_MASK",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/SIZE",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/SLOT",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/USE",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/environment.bz2",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/extract-test-1.2.3.ebuild",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/license.yaml",
            "build/foo/var/db/pkg/virtual/extract-test-1.2.3/repository"
        ]
    );
    Ok(())
}

#[test]
fn staged_image_for_host() -> Result<()> {
    let out_dir = run_extract_package(".image", "", true)?;
    assert_eq!(
        list_files_under(out_dir.path())?,
        vec![
            ".image/bin/helloworld",
            ".image/lib64/helloworld", // rewrote lib->lib64
            ".image/usr/bin/helloworld",
            ".image/usr/lib64/helloworld", // rewrote lib->lib64
            "var/db/pkg/virtual/extract-test-1.2.3/BUILD_TIME",
            "var/db/pkg/virtual/extract-test-1.2.3/CATEGORY",
            "var/db/pkg/virtual/extract-test-1.2.3/CBUILD",
            "var/db/pkg/virtual/extract-test-1.2.3/CC",
            "var/db/pkg/virtual/extract-test-1.2.3/CFLAGS",
            "var/db/pkg/virtual/extract-test-1.2.3/CHOST",
            "var/db/pkg/virtual/extract-test-1.2.3/CONTENTS",
            "var/db/pkg/virtual/extract-test-1.2.3/COUNTER",
            "var/db/pkg/virtual/extract-test-1.2.3/CXX",
            "var/db/pkg/virtual/extract-test-1.2.3/CXXFLAGS",
            "var/db/pkg/virtual/extract-test-1.2.3/DEFINED_PHASES",
            "var/db/pkg/virtual/extract-test-1.2.3/EAPI",
            "var/db/pkg/virtual/extract-test-1.2.3/FEATURES",
            "var/db/pkg/virtual/extract-test-1.2.3/IUSE",
            "var/db/pkg/virtual/extract-test-1.2.3/IUSE_EFFECTIVE",
            "var/db/pkg/virtual/extract-test-1.2.3/KEYWORDS",
            "var/db/pkg/virtual/extract-test-1.2.3/LDFLAGS",
            "var/db/pkg/virtual/extract-test-1.2.3/PF",
            "var/db/pkg/virtual/extract-test-1.2.3/PKG_INSTALL_MASK",
            "var/db/pkg/virtual/extract-test-1.2.3/SIZE",
            "var/db/pkg/virtual/extract-test-1.2.3/SLOT",
            "var/db/pkg/virtual/extract-test-1.2.3/USE",
            "var/db/pkg/virtual/extract-test-1.2.3/environment.bz2",
            "var/db/pkg/virtual/extract-test-1.2.3/extract-test-1.2.3.ebuild",
            "var/db/pkg/virtual/extract-test-1.2.3/license.yaml",
            "var/db/pkg/virtual/extract-test-1.2.3/repository"
        ]
    );
    Ok(())
}
