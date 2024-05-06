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

const TEST_CPF: &str = "foo/bar-1.2.3";
const TEST_PF: &str = "bar-1.2.3";
const TEST_PN: &str = "bar";

fn run_drive_binary_package(
    root_dir: &Path,
    image_dir: &Path,
    extra_options: &[&str],
    phases: &[&str],
) -> Result<Output> {
    let temp_dir = TempDir::new()?;
    let temp_dir = temp_dir.path();
    let r = Runfiles::create()?;
    let program_path = runfiles::rlocation!(
        r,
        "cros/bazel/portage/bin/drive_binary_package/drive_binary_package.sh"
    );
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
        .arg(TEST_CPF)
        .args(extra_options)
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
fn create_test_vdb(root_dir: &Path, extra_environment: &str) -> Result<()> {
    let vdb_dir = get_vdb_dir(root_dir, TEST_CPF);
    std::fs::create_dir_all(&vdb_dir)?;

    std::fs::write(vdb_dir.join("repository"), "chromiumos")?;

    let mut environment = File::create(vdb_dir.join("environment.raw"))?;
    write!(
        &mut environment,
        "EAPI=7\nPF={}\nPN={}\nSLOT=0\n{}",
        TEST_PF, TEST_PN, extra_environment
    )?;

    Ok(())
}

/// Verify the case where no hook is defined.
#[test]
fn no_hooks() -> Result<()> {
    let root_dir = TempDir::new()?;
    let root_dir = root_dir.path();
    let image_dir = &root_dir.join(".image");
    std::fs::create_dir_all(image_dir)?;

    create_test_vdb(root_dir, "")?;
    run_drive_binary_package(root_dir, image_dir, &[], &["setup", "preinst", "postinst"])?;

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
    let output =
        run_drive_binary_package(root_dir, image_dir, &[], &["setup", "preinst", "postinst"])?;

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
        let output = run_drive_binary_package(root_dir, image_dir, &[], &[phase])?;

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
    run_drive_binary_package(root_dir, image_dir, &[], &["setup", "preinst", "postinst"])?;

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

    create_test_vdb(root_dir, &test_script)?;
    run_drive_binary_package(root_dir, image_dir, &[], &["setup", "preinst", "postinst"])?;

    Ok(())
}

// Verify the option to not save the environment.
#[test]
fn no_clobber() -> Result<()> {
    let root_dir = TempDir::new()?;
    let root_dir = root_dir.path();
    let image_dir = &root_dir.join(".image");
    std::fs::create_dir_all(image_dir)?;

    create_test_vdb(
        root_dir,
        r#"
    pkg_setup() {
        export MY_"NEW"_VARIABLE=defined
    }
    "#,
    )?;
    run_drive_binary_package(root_dir, image_dir, &["-n"], &["setup"])?;

    let environment =
        std::fs::read_to_string(get_vdb_dir(root_dir, TEST_CPF).join("environment.raw"))?;
    assert!(!environment.contains("MY_NEW_VARIABLE"));

    Ok(())
}

// Verify that invalid options result in internal errors.
#[test]
fn invalid_options() -> Result<()> {
    let root_dir = TempDir::new()?;
    let root_dir = root_dir.path();
    let image_dir = &root_dir.join(".image");
    std::fs::create_dir_all(image_dir)?;

    create_test_vdb(root_dir, "")?;
    let error = run_drive_binary_package(root_dir, image_dir, &["-X"], &["setup"]).unwrap_err();
    assert!(error.to_string().contains("INTERNAL ERROR"), "{error}");

    Ok(())
}

// Verify that all ebuild-specific functions defined in PMS are provided.
#[test]
fn ebuild_functions() -> Result<()> {
    // All ebuild-specific functions defined in PMS 8.
    // https://projects.gentoo.org/pms/8/pms.html#x1-12000012.3
    const FUNCS: &[&str] = &[
        // 12.3.1 Failure behavior and related commands
        "nonfatal",
        // 12.3.3 Sandbox commands
        "addread",
        "addwrite",
        "addpredict",
        "adddeny",
        // 12.3.4 Package manager query commands
        "has_version",
        "best_version",
        // 12.3.5 Output commands
        "einfo",
        "einfon",
        "elog",
        "ewarn",
        "eqawarn",
        "eerror",
        "ebegin",
        "eend",
        // 12.3.6 Error commands
        "die",
        "assert",
        // 12.3.7 Patch commands
        "eapply",
        "eapply_user",
        // 12.3.8 Build commands
        "econf",
        "emake",
        "einstall",
        // 12.3.9 Installation commands
        "dobin",
        "doconfd",
        "dodir",
        "dodoc",
        "doenvd",
        "doexe",
        "dohard",
        "doheader",
        "dohtml",
        "doinfo",
        "doinitd",
        "doins",
        "dolib.a",
        "dolib.so",
        "dolib",
        "doman",
        "domo",
        "dosbin",
        "dosym",
        "fowners",
        "fperms",
        "keepdir",
        "newbin",
        "newconfd",
        "newdoc",
        "newenvd",
        "newexe",
        "newheader",
        "newinitd",
        "newins",
        "newlib.a",
        "newlib.so",
        "newman",
        "newsbin",
        // 12.3.10 Commands affecting install destinations
        "into",
        "insinto",
        "exeinto",
        "docinto",
        "insopts",
        "diropts",
        "exeopts",
        "libopts",
        // 12.3.11 Commands controlling manipulation of files in the staging area
        "docompress",
        "dostrip",
        // 12.3.12 USE list functions
        "use",
        "usev",
        "useq",
        "use_with",
        "use_enable",
        "usex",
        "in_iuse",
        // 12.3.13 Text list functions
        "has",
        "hasv",
        "hasq",
        // 12.3.14 Version manipulation and comparison commands
        // These functions are provided in the binary package's environment.
        // "ver_cut",
        // "ver_rs",
        // "ver_test",
        // 12.3.15 Misc commands
        "dosed",
        "unpack",
        // "inherit", // Already resolved in the binary package's environment.
        "default",
        "einstalldocs",
        "get_libdir",
        // 12.3.16 Debug commands
        "debug-print",
        "debug-print-function",
        "debug-print-section",
    ];

    let root_dir = TempDir::new()?;
    let root_dir = root_dir.path();
    let image_dir = &root_dir.join(".image");
    std::fs::create_dir_all(image_dir)?;

    let extra_environment = r#"
    pkg_setup() {
        for name in %FUNCS%; do
            if ! type -t "${name}" > /dev/null 2>&1; then
                echo "FAIL: ${name} is not provided!"
                exit 1
            fi
        done
    }
    "#
    .to_string()
    .replace("%FUNCS%", &FUNCS.join(" "));

    create_test_vdb(root_dir, &extra_environment)?;
    run_drive_binary_package(root_dir, image_dir, &["-n"], &["setup"])?;
    Ok(())
}
