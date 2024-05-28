// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use container::ContainerSettings;
use durabletree::DurableTree;
use fileutil::{resolve_symlink_forest, SafeTempDirBuilder};
use runfiles::Runfiles;
use serde::Serialize;
use testutil::{compare_with_golden_data, describe_tree, fakefs_chown};
use walkdir::WalkDir;

use std::fs::set_permissions;
use std::fs::{File, Permissions};
use std::io::BufWriter;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

// These tests need to run in an user namespace so that the current process UID/GID are 0.
#[used]
#[link_section = ".init_array"]
static _CTOR: extern "C" fn() = ::testutil::ctor_enter_mount_namespace;

const BASE_DIR: &str = "cros/bazel/portage/bin/create_interface_layer";

fn lookup_runfile(runfile_path: impl AsRef<Path>) -> Result<PathBuf> {
    let r = Runfiles::create()?;
    let full_path = runfiles::rlocation!(r, runfile_path.as_ref());
    if !full_path.try_exists()? {
        bail!("{full_path:?} does not exist");
    }

    Ok(full_path)
}

/// Copies the src into dest and performs additional modifications to it.
///
/// We can't commit chmod and chown changes into git, so we recreate them at
/// runtime.
fn setup_input_layer(src: &Path, sysroot: &Path, dest: &Path) -> Result<()> {
    Command::new("cp").arg("-ra").arg(src).arg(dest).status()?;

    let sysroot = dest.join(
        sysroot
            .strip_prefix("/")
            .context("sysroot must be absolute")?,
    );

    // Copy the group permission bits to other so that we can replicate a more
    // relaxed umask.
    // i.e., 640 -> 644, 750 -> 755.
    for entry in WalkDir::new(dest) {
        let entry = entry?;
        let metadata = entry
            .metadata()
            .with_context(|| format!("stat {:?}", entry.path()))?;

        let orig_mode = metadata.permissions().mode();
        let mode = orig_mode | ((orig_mode & 0o070) >> 3);

        if mode != orig_mode {
            set_permissions(entry.path(), Permissions::from_mode(mode))
                .with_context(|| format!("chmod {:o} {:?}", mode, entry.path()))?;
        }
    }

    set_permissions(sysroot.join("var/tmp"), Permissions::from_mode(0o1777))
        .with_context(|| format!("chmod 1777 {}", sysroot.join("var/tmp").display()))?;
    fakefs_chown("400:600", &sysroot.join("var/tmp/portage"))?;
    fakefs_chown("400:0", &sysroot.join("var/tmp/portage/.keep"))?;
    set_permissions(
        sysroot.join("var/tmp/portage/.keep"),
        Permissions::from_mode(0o444),
    )?;

    // Make it readonly to ensure permissions are restored last.
    set_permissions(sysroot.join("var"), Permissions::from_mode(0o555))?;

    Ok(())
}

fn durabletree_to_manifest(mutable_base_dir: &Path, input: &Path, manifest: &Path) -> Result<()> {
    DurableTree::cool_down_for_testing(input)?;

    // Serialize the tree into a json file so we can compare against the golden file.
    let mut settings = ContainerSettings::new();
    settings.set_mutable_base_dir(mutable_base_dir);
    settings.push_layer(input)?;
    let input = settings.mount()?;

    let tree = describe_tree(input.path())?;

    let mut serializer = serde_json::Serializer::pretty(BufWriter::new(File::create(manifest)?));

    tree.serialize(&mut serializer)?;

    Ok(())
}

fn compare_input_layer_to_golden(
    input_layer: &Path,
    sysroot: &Path,
    golden_file: &Path,
) -> Result<()> {
    let tmp_dir = SafeTempDirBuilder::new()
        .base_dir(&PathBuf::from(
            std::env::var("TEST_TMPDIR").context("TEST_TMPDIR is not set")?,
        ))
        .build()?;
    let tmp_dir = tmp_dir.path();

    let output = tmp_dir.join("output");

    let input_layer = resolve_symlink_forest(&lookup_runfile(input_layer)?)?;

    let real_input_layer = tmp_dir.join("input");
    setup_input_layer(&input_layer, sysroot, &real_input_layer)?;

    let status = Command::new(lookup_runfile(
        Path::new(BASE_DIR).join("create_interface_layer"),
    )?)
    .arg("--sysroot")
    .arg(sysroot)
    .arg("--layer")
    .arg(lookup_runfile("cros/bazel/portage/sdk/sdk_from_archive")?)
    .arg("--input")
    .arg(&real_input_layer)
    .arg("--output")
    .arg(&output)
    .status()?;

    assert!(status.success());

    let manifest_path = tmp_dir.join("manifest.json");
    durabletree_to_manifest(tmp_dir, &output, &manifest_path)?;

    // Use the following to regenerate the golden data:
    // ALCHEMY_REGENERATE_GOLDEN=1 bazel run :integration_tests_tests/run_binary
    compare_with_golden_data(&manifest_path, golden_file)?;

    Ok(())
}

#[test]
fn test_board_sysroot() -> Result<()> {
    compare_input_layer_to_golden(
        &Path::new(BASE_DIR).join("testdata/input"),
        Path::new("/build/board"),
        Path::new("bazel/portage/bin/create_interface_layer/testdata/golden/board/manifest.json"),
    )?;
    Ok(())
}

#[test]
fn test_host_sysroot() -> Result<()> {
    compare_input_layer_to_golden(
        // Instead of creating a new input directory, we just use the board's
        // sysroot as the host sysroot.
        &Path::new(BASE_DIR).join("testdata/input/build/board"),
        Path::new("/"),
        Path::new("bazel/portage/bin/create_interface_layer/testdata/golden/host/manifest.json"),
    )?;
    Ok(())
}
