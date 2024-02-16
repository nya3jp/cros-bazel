// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::File,
    io::Write,
    path::Path,
    process::{Command, Output},
};

use anyhow::{ensure, Result};
use runfiles::Runfiles;
use tempfile::TempDir;
use vdb::get_vdb_dir;

fn run_drive_binary_package(
    root_dir: &Path,
    image_dir: &Path,
    cpf: &str,
    phases: &[&str],
) -> Result<Output> {
    let temp_dir = TempDir::new()?;
    let temp_dir = temp_dir.path();
    let runfiles = Runfiles::create()?;
    let program_path =
        runfiles.rlocation("cros/bazel/portage/bin/drive_binary_package/drive_binary_package.sh");
    let path_with_fakes = format!(
        "bazel/portage/bin/drive_binary_package/testdata/fakes:{}",
        std::env::var("PATH")?
    );
    let output = Command::new(program_path)
        .arg("-r")
        .arg(root_dir)
        .arg("-d")
        .arg(image_dir)
        .arg("-t")
        .arg(temp_dir)
        .arg("-p")
        .arg(cpf)
        .args(phases)
        .env("PATH", path_with_fakes)
        .output()?;
    ensure!(
        output.status.success(),
        "drive_binary_package failed: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    Ok(output)
}

/// Creates a VDB for testing with the specified environment.
fn create_test_vdb(root_dir: &Path, cpf: &str, extra_environment: &str) -> Result<()> {
    let vdb_dir = get_vdb_dir(root_dir, cpf);
    std::fs::create_dir_all(&vdb_dir)?;

    std::fs::write(vdb_dir.join("repository"), "chromiumos")?;

    let mut environment = File::create(vdb_dir.join("environment.raw"))?;
    write!(
        &mut environment,
        "EAPI=7\nPF={}\n{}",
        cpf.split_once('/').unwrap().1,
        extra_environment
    )?;

    Ok(())
}

const TEST_CPF: &str = "foo/bar-1.2.3";

/// Verify the case where no hook is defined.
#[test]
fn no_hooks() -> Result<()> {
    let root_dir = TempDir::new()?;
    let root_dir = root_dir.path();
    let image_dir = &root_dir.join(".image");
    std::fs::create_dir_all(image_dir)?;

    create_test_vdb(root_dir, TEST_CPF, "")?;
    run_drive_binary_package(
        root_dir,
        image_dir,
        TEST_CPF,
        &["setup", "preinst", "postinst"],
    )?;

    Ok(())
}

// Verify that messages printed by hooks are captured.
#[test]
fn print_messages() -> Result<()> {
    let root_dir = TempDir::new()?;
    let root_dir = root_dir.path();
    let image_dir = &root_dir.join(".image");
    std::fs::create_dir_all(image_dir)?;

    create_test_vdb(
        root_dir,
        TEST_CPF,
        r#"
    pkg_setup() {
        echo "this is pkg_setup"
    }
    pkg_preinst() {
        echo "this is pkg_preinst"
    }
    pkg_postinst() {
        echo "this is pkg_postinst"
    }
    "#,
    )?;
    let output = run_drive_binary_package(
        root_dir,
        image_dir,
        TEST_CPF,
        &["setup", "preinst", "postinst"],
    )?;

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("this is pkg_setup"), "stdout:\n{}", stdout);
    assert!(
        stdout.contains("this is pkg_preinst"),
        "stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("this is pkg_postinst"),
        "stdout:\n{}",
        stdout
    );

    Ok(())
}

// Verify that the environment is kept across hooks.
#[test]
fn keep_environment() -> Result<()> {
    let root_dir = TempDir::new()?;
    let root_dir = root_dir.path();
    let image_dir = &root_dir.join(".image");
    std::fs::create_dir_all(image_dir)?;

    create_test_vdb(
        root_dir,
        TEST_CPF,
        r#"
    MY_COUNTER=0
    pkg_setup() {
        echo "MY_COUNTER=${MY_COUNTER}"
        MY_COUNTER=1
    }
    pkg_preinst() {
        echo "MY_COUNTER=${MY_COUNTER}"
        MY_COUNTER=2
    }
    pkg_postinst() {
        echo "MY_COUNTER=${MY_COUNTER}"
        MY_COUNTER=3
    }
    "#,
    )?;

    for (phase, expected_counter) in [("setup", "0"), ("preinst", "1"), ("postinst", "2")] {
        let output = run_drive_binary_package(root_dir, image_dir, TEST_CPF, &[phase])?;

        let stdout = String::from_utf8(output.stdout)?;
        let expect = format!("MY_COUNTER={}", expected_counter);
        assert!(
            stdout.contains(&expect),
            "stdout:\n{}\nwanted: {}",
            stdout,
            &expect,
        );
    }
    Ok(())
}

// Verify the ability to modify the file system via defined variables.
#[test]
fn modify_file_system() -> Result<()> {
    let root_dir = TempDir::new()?;
    let root_dir = root_dir.path();
    let image_dir = &root_dir.join(".image");
    std::fs::create_dir_all(image_dir)?;

    create_test_vdb(
        root_dir,
        TEST_CPF,
        r#"
    pkg_setup() {
        touch "${ROOT}/pkg_setup"
    }
    pkg_preinst() {
        touch "${ROOT}/pkg_preinst"
        touch "${D}/pkg_preinst_d"
    }
    pkg_postinst() {
        touch "${ROOT}/pkg_postinst"
    }
    "#,
    )?;
    run_drive_binary_package(
        root_dir,
        image_dir,
        TEST_CPF,
        &["setup", "preinst", "postinst"],
    )?;

    assert!(root_dir.join("pkg_setup").exists());
    assert!(root_dir.join("pkg_preinst").exists());
    assert!(root_dir.join("pkg_postinst").exists());
    assert!(image_dir.join("pkg_preinst_d").exists());

    Ok(())
}

// Verify the ability to modify the file system via defined variables.
#[test]
fn ebuild_function_tests() -> Result<()> {
    let root_dir = TempDir::new()?;
    let root_dir = root_dir.path();
    let image_dir = &root_dir.join(".image");
    std::fs::create_dir_all(image_dir)?;

    let test_script = std::fs::read_to_string(
        "bazel/portage/bin/drive_binary_package/testdata/ebuild_function_tests.sh",
    )?;

    create_test_vdb(root_dir, TEST_CPF, &test_script)?;
    run_drive_binary_package(
        root_dir,
        image_dir,
        TEST_CPF,
        &["setup", "preinst", "postinst"],
    )?;

    Ok(())
}
