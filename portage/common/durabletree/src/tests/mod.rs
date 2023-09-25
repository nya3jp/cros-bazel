// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod testutil;

use anyhow::Result;
use fileutil::SafeTempDir;
use std::{
    fs::{create_dir, set_permissions, File},
    os::unix::{fs::symlink, prelude::*},
    path::PathBuf,
    process::Command,
};
use walkdir::WalkDir;

use crate::{
    consts::{MODE_MASK, RAW_DIR_NAME},
    tests::testutil::{
        describe_tree, simple_dir, simple_file, CommandRunOk, FileDescription, EMPTY_HASH,
    },
    DurableTree,
};

// Run unit tests in a mount namespace.
#[used]
#[link_section = ".init_array"]
static _CTOR: extern "C" fn() = ::testutil::ctor_enter_mount_namespace;

// Tests `DurableTree::try_exists`.
#[test]
fn exists() -> Result<()> {
    let dir = SafeTempDir::new()?;
    let dir = dir.path();

    assert!(!DurableTree::try_exists(dir)?);

    DurableTree::convert(dir)?;

    assert!(DurableTree::try_exists(dir)?);

    Ok(())
}

// Tries converting and expanding a durable tree from an empty directory.
#[test]
fn empty() -> Result<()> {
    let dir = SafeTempDir::new()?;
    let dir = dir.path();

    set_permissions(dir, PermissionsExt::from_mode(0o750))?;
    assert_eq!(0o750, dir.metadata()?.mode() & MODE_MASK);

    DurableTree::convert(dir)?;
    DurableTree::cool_down_for_testing(dir)?;
    let tree = DurableTree::expand(dir)?;

    let files: Vec<Vec<FileDescription>> = tree
        .layers()
        .into_iter()
        .map(describe_tree)
        .collect::<Result<_>>()?;

    assert_eq!(files, Vec::<Vec<FileDescription>>::new());

    Ok(())
}

// Tries converting and expanding a durable tree from a simple directory.
#[test]
fn simple() -> Result<()> {
    let dir = SafeTempDir::new()?;
    let dir = dir.path();

    // ./         - 0755
    //   a.txt    - 0666
    //   dir/     - 0775
    //     b.txt  - 0700
    //     link  -> ../z.txt
    set_permissions(dir, PermissionsExt::from_mode(0o755))?;
    File::create(dir.join("a.txt"))?.set_permissions(PermissionsExt::from_mode(0o666))?;
    create_dir(dir.join("dir"))?;
    set_permissions(dir.join("dir"), PermissionsExt::from_mode(0o755))?;
    File::create(dir.join("dir/b.txt"))?.set_permissions(PermissionsExt::from_mode(0o700))?;
    symlink("../z.txt", dir.join("dir/link"))?;

    DurableTree::convert(dir)?;
    DurableTree::cool_down_for_testing(dir)?;
    let tree = DurableTree::expand(dir)?;

    let files: Vec<Vec<FileDescription>> = tree
        .layers()
        .into_iter()
        .map(describe_tree)
        .collect::<Result<_>>()?;

    assert_eq!(
        files,
        vec![
            vec![
                simple_dir("", 0o755),
                simple_dir("dir", 0o755),
                FileDescription::Symlink {
                    path: PathBuf::from("dir/link"),
                    mode: 0o777,
                    target: PathBuf::from("../z.txt"),
                },
            ],
            vec![
                simple_dir("", 0o755),
                simple_file("a.txt", 0o666, EMPTY_HASH),
                simple_dir("dir", 0o755),
                simple_file("dir/b.txt", 0o700, EMPTY_HASH),
            ],
        ],
    );

    Ok(())
}

// Checks the case with special files.
#[test]
fn preserve_special_files() -> Result<()> {
    let dir = SafeTempDir::new()?;
    let dir = dir.path();

    set_permissions(dir, PermissionsExt::from_mode(0o755))?;
    // Whiteout file used by overlayfs.
    Command::new("mknod")
        .args([
            "--mode=0644",
            &dir.join("whiteout").to_string_lossy(),
            "c",
            "0",
            "0",
        ])
        .run_ok()?;

    DurableTree::convert(dir)?;
    DurableTree::cool_down_for_testing(dir)?;
    let tree = DurableTree::expand(dir)?;

    let files: Vec<Vec<FileDescription>> = tree
        .layers()
        .into_iter()
        .map(describe_tree)
        .collect::<Result<_>>()?;

    assert_eq!(
        files,
        vec![
            vec![
                simple_dir("", 0o755),
                FileDescription::Char {
                    path: PathBuf::from("whiteout"),
                    mode: 0o644,
                    rdev: 0,
                },
            ],
            vec![simple_dir("", 0o755)],
        ],
    );

    Ok(())
}

// Checks the case with special mode bits (sticky/setuid/setgid).
#[test]
fn preserve_special_modes() -> Result<()> {
    let dir = SafeTempDir::new()?;
    let dir = dir.path();

    // ./         -  0700
    //   tmp/     - 01777
    //     setgid - 02700
    //     setuid - 04700
    set_permissions(dir, PermissionsExt::from_mode(0o700))?;
    create_dir(dir.join("tmp"))?;
    set_permissions(dir.join("tmp"), PermissionsExt::from_mode(0o1777))?;
    File::create(dir.join("tmp/setgid"))?.set_permissions(PermissionsExt::from_mode(0o2700))?;
    File::create(dir.join("tmp/setuid"))?.set_permissions(PermissionsExt::from_mode(0o4700))?;

    DurableTree::convert(dir)?;
    DurableTree::cool_down_for_testing(dir)?;
    let tree = DurableTree::expand(dir)?;

    let files: Vec<Vec<FileDescription>> = tree
        .layers()
        .into_iter()
        .map(describe_tree)
        .collect::<Result<_>>()?;

    assert_eq!(
        files,
        vec![vec![
            simple_dir("", 0o700),
            simple_dir("tmp", 0o1777),
            simple_file("tmp/setgid", 0o2700, EMPTY_HASH),
            simple_file("tmp/setuid", 0o4700, EMPTY_HASH),
        ]],
    );

    Ok(())
}

// Checks that user xattrs are preserved.
#[test]
fn preserve_user_xattrs() -> Result<()> {
    let dir = SafeTempDir::new()?;
    let dir = dir.path();

    set_permissions(dir, PermissionsExt::from_mode(0o700))?;
    xattr::set(dir, "user.aaa", "xxx".as_bytes())?;
    xattr::set(dir, "user.bbb", "yyy".as_bytes())?;
    xattr::set(dir, "user.ccc", "zzz".as_bytes())?;

    let file = dir.join("file");
    File::create(&file)?.set_permissions(PermissionsExt::from_mode(0o644))?;
    xattr::set(&file, "user.aaa", &111_u32.to_be_bytes())?;
    xattr::set(&file, "user.bbb", &222_u32.to_be_bytes())?;
    xattr::set(&file, "user.ccc", &333_u32.to_be_bytes())?;

    DurableTree::convert(dir)?;
    DurableTree::cool_down_for_testing(dir)?;
    let tree = DurableTree::expand(dir)?;

    let files: Vec<Vec<FileDescription>> = tree
        .layers()
        .into_iter()
        .map(describe_tree)
        .collect::<Result<_>>()?;

    assert_eq!(
        files,
        vec![vec![
            FileDescription::Dir {
                path: PathBuf::from(""),
                mode: 0o700,
                user_xattrs: [
                    ("user.aaa".to_owned(), Vec::from("xxx")),
                    ("user.bbb".to_owned(), Vec::from("yyy")),
                    ("user.ccc".to_owned(), Vec::from("zzz")),
                ]
                .into(),
            },
            FileDescription::File {
                path: PathBuf::from("file"),
                mode: 0o644,
                hash: EMPTY_HASH.to_owned(),
                user_xattrs: [
                    ("user.aaa".to_owned(), Vec::from(111_u32.to_be_bytes())),
                    ("user.bbb".to_owned(), Vec::from(222_u32.to_be_bytes())),
                    ("user.ccc".to_owned(), Vec::from(333_u32.to_be_bytes())),
                ]
                .into(),
            },
        ]],
    );

    Ok(())
}

// Checks that empty directories are restored.
#[test]
fn restore_empty_dirs() -> Result<()> {
    let dir = SafeTempDir::new()?;
    let dir = dir.path();

    set_permissions(dir, PermissionsExt::from_mode(0o750))?;
    create_dir(dir.join("aaa"))?;
    set_permissions(dir.join("aaa"), PermissionsExt::from_mode(0o750))?;
    create_dir(dir.join("aaa/bbb"))?;
    set_permissions(dir.join("aaa/bbb"), PermissionsExt::from_mode(0o750))?;

    DurableTree::convert(dir)?;
    DurableTree::cool_down_for_testing(dir)?;

    // Remove empty directories.
    fileutil::remove_dir_all_with_chmod(&dir.join(RAW_DIR_NAME).join("aaa"))?;

    let tree = DurableTree::expand(dir)?;

    let files: Vec<Vec<FileDescription>> = tree
        .layers()
        .into_iter()
        .map(describe_tree)
        .collect::<Result<_>>()?;

    assert_eq!(
        files,
        vec![vec![
            simple_dir("", 0o750),
            simple_dir("aaa", 0o750),
            simple_dir("aaa/bbb", 0o750),
        ]],
    );

    Ok(())
}

// Ensure that restoration happens only once.
#[test]
fn restore_once() -> Result<()> {
    let dir = SafeTempDir::new()?;
    let dir = dir.path();
    let raw_dir = dir.join(RAW_DIR_NAME);

    set_permissions(dir, PermissionsExt::from_mode(0o750))?;

    DurableTree::convert(dir)?;
    DurableTree::cool_down_for_testing(dir)?;

    // Expansion once.
    DurableTree::expand(dir)?;
    assert_eq!(describe_tree(&raw_dir)?, vec![simple_dir("", 0o750)]);

    // If we change the permission of the raw directory here, it won't be
    // restored again.
    set_permissions(&raw_dir, PermissionsExt::from_mode(0o700))?;

    DurableTree::expand(dir)?;
    assert_eq!(describe_tree(&raw_dir)?, vec![simple_dir("", 0o700)]);

    // By cooling down, restoration happens again.
    DurableTree::cool_down_for_testing(dir)?;

    DurableTree::expand(dir)?;
    assert_eq!(describe_tree(&raw_dir)?, vec![simple_dir("", 0o750)]);

    Ok(())
}

// Checks that the same input directory produces a bit-identical durable tree.
#[test]
fn reproducible() -> Result<()> {
    let dir = SafeTempDir::new()?;
    let dir = dir.path();

    set_permissions(dir, PermissionsExt::from_mode(0o755))?;

    // Try to cover as many file types.
    // Regular file:
    {
        let path = dir.join("file");
        File::create(&path)?.set_permissions(PermissionsExt::from_mode(0o644))?;
        xattr::set(&path, "user.aaa", &111_u32.to_be_bytes())?;
        xattr::set(&path, "user.bbb", &222_u32.to_be_bytes())?;
        xattr::set(&path, "user.ccc", &1333_u32.to_be_bytes())?;
    }
    // Directory:
    {
        let path = dir.join("dir");
        create_dir(&path)?;
        set_permissions(&path, PermissionsExt::from_mode(0o755))?;
        xattr::set(&path, "user.xxx", &111_u32.to_be_bytes())?;
        xattr::set(&path, "user.yyy", &222_u32.to_be_bytes())?;
        xattr::set(&path, "user.zzz", &1333_u32.to_be_bytes())?;
    }
    // Symlink:
    symlink("/path/to/something", dir.join("symlink"))?;
    // Whiteout file:
    Command::new("mknod")
        .args([
            "--mode=0644",
            &dir.join("whiteout").to_string_lossy(),
            "c",
            "0",
            "0",
        ])
        .run_ok()?;

    DurableTree::convert(dir)?;
    DurableTree::cool_down_for_testing(dir)?;

    let files = describe_tree(dir)?;

    assert_eq!(
        files,
        vec![
            simple_dir("", 0o555),
            simple_file("DURABLE_TREE", 0o555, EMPTY_HASH),
            simple_file(
                "extra.tar.zst",
                0o555,
                "723dd57d71df40fd207807e1ea96740c535e8ad160cb040ba555ae8273cba8e8"
            ),
            simple_file(
                "manifest.json",
                0o555,
                "8893c37c3a03a2b39d9979670f299a564a25fbaa1239eacacf5d5831e5df9ef1"
            ),
            simple_dir("raw", 0o555),
            simple_dir("raw/dir", 0o555),
            simple_file("raw/file", 0o555, EMPTY_HASH),
        ]
    );

    Ok(())
}

// Tries converting and expanding a durable tree involving files/directories
// whose permissions are set inaccessible.
#[test]
fn inaccessible_files() -> Result<()> {
    let dir = SafeTempDir::new()?;
    let dir = dir.path();

    // ./         - 0700
    //   dir1/    - 0000
    //     dir2/  - 0000
    //       file - 0000
    let dir1 = dir.join("dir1");
    create_dir(&dir1)?;
    let dir2 = dir1.join("dir2");
    create_dir(&dir2)?;
    File::create(dir2.join("file"))?.set_permissions(PermissionsExt::from_mode(0o0))?;
    set_permissions(&dir2, PermissionsExt::from_mode(0o0))?;
    set_permissions(&dir1, PermissionsExt::from_mode(0o0))?;
    set_permissions(dir, PermissionsExt::from_mode(0o700))?;

    DurableTree::convert(dir)?;

    // Ensure that all files under the raw directory has the permission 0755.
    for entry in WalkDir::new(dir.join("raw")) {
        let entry = entry?;
        let mode = entry.metadata()?.mode() & MODE_MASK;
        assert_eq!(
            mode,
            0o755,
            "Unexpected permission: {}: got 0{:03o}, want 0755",
            entry.path().display(),
            mode
        );
    }

    DurableTree::cool_down_for_testing(dir)?;
    let tree = DurableTree::expand(dir)?;

    let files: Vec<Vec<FileDescription>> = tree
        .layers()
        .into_iter()
        .map(describe_tree)
        .collect::<Result<_>>()?;

    assert_eq!(
        files,
        vec![vec![
            simple_dir("", 0o700),
            simple_dir("dir1", 0),
            simple_dir("dir1/dir2", 0),
            simple_file("dir1/dir2/file", 0, EMPTY_HASH),
        ]],
    );

    Ok(())
}
