// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// This file provides "unit tests" of fast_install_packages. They run
// `fast_install_packages` and `extract_package` as subprocesses, instead of
// invoking its implementations with function calls, because its CLI presents
// the public API surface suitable for testing.

use anyhow::{ensure, Result};
use bytes::BufMut;
use bzip2::{read::BzEncoder, Compression};
use container::ContainerSettings;
use durabletree::DurableTree;
use fileutil::{SafeTempDir, SafeTempDirBuilder};
use itertools::Itertools;
use processes::locate_system_binary;
use runfiles::Runfiles;
use std::{
    fs::{OpenOptions, Permissions},
    io::{Read, Write},
    os::unix::prelude::PermissionsExt,
    path::Path,
    process::Command,
};
use tempfile::{NamedTempFile, TempDir};
use vdb::get_vdb_dir;

// Run unit tests in a mount namespace to use durable trees.
#[used]
#[link_section = ".init_array"]
static _CTOR: extern "C" fn() = ::testutil::ctor_enter_mount_namespace;

static DEFAULT_SLOT: &str = "0/0";

/// Creates a temporary directory in the execroot (".").
fn temp_dir_for_testing() -> Result<SafeTempDir> {
    SafeTempDirBuilder::new().base_dir(Path::new(".")).build()
}

/// Runs `extract_package` program. The results are saved to a temporary
/// directory in the execroot as a durable tree, and its path is returned as
/// [`SafeTempDir`].
fn run_extract_package(
    binary_package: &Path,
    image_prefix: &str,
    vdb_prefix: &str,
    host: bool,
) -> Result<SafeTempDir> {
    let out_dir = temp_dir_for_testing()?;

    let runfiles = Runfiles::create()?;
    let program_path = runfiles.rlocation("cros/bazel/portage/bin/extract_package/extract_package");
    let status = Command::new(program_path)
        .arg("--input-binary-package")
        .arg(binary_package)
        .arg("--output-directory")
        .arg(out_dir.path())
        .arg("--image-prefix")
        .arg(image_prefix)
        .arg("--vdb-prefix")
        .arg(vdb_prefix)
        .args(if host { &["--host"][..] } else { &[][..] })
        .status()?;
    ensure!(status.success(), "extract_package failed: {:?}", status);

    // Clear the hot state of output durable trees for convenience.
    DurableTree::cool_down_for_testing(out_dir.path())?;

    Ok(out_dir)
}

/// Specifies information needed to install a package.
struct InstallSpec<'a> {
    pub input_binary_package: &'a Path,
    pub input_installed_contents_dir: &'a Path,
    pub input_staged_contents_dir: &'a Path,
    pub output_preinst_dir: &'a Path,
    pub output_postinst_dir: &'a Path,
}

/// Runs `fast_install_packages` program.
fn run_fast_install_packages(
    container_image_path: &Path,
    root_dir: &Path,
    specs: &[&InstallSpec],
) -> Result<()> {
    let runfiles = Runfiles::create()?;

    let program_path =
        runfiles.rlocation("cros/bazel/portage/bin/fast_install_packages/fast_install_packages");

    let mut command = Command::new(program_path);
    command
        .arg("--layer")
        .arg(container_image_path)
        .arg("--root-dir")
        .arg(root_dir);
    for spec in specs {
        command.arg(format!(
            "--install={},{},{},{},{}",
            spec.input_binary_package.display(),
            spec.input_installed_contents_dir.display(),
            spec.input_staged_contents_dir.display(),
            spec.output_preinst_dir.display(),
            spec.output_postinst_dir.display()
        ));
    }
    let status = command.status()?;
    ensure!(
        status.success(),
        "fast_install_packages failed: {:?}",
        status
    );

    // Clear the hot state of output durable trees for convenience.
    for spec in specs {
        DurableTree::cool_down_for_testing(spec.output_preinst_dir)?;
        DurableTree::cool_down_for_testing(spec.output_postinst_dir)?;
    }

    Ok(())
}

struct InstalledGuard {
    pub installed_contents_dir: SafeTempDir,
    pub staged_contents_dir: SafeTempDir,
    pub preinst_dir: SafeTempDir,
    pub postinst_dir: SafeTempDir,
}

/// Installs zero or more packages into a container with fast_install_packages.
fn fast_install_packages(
    settings: &mut ContainerSettings,
    binary_packages: &[&Path],
    root_dir: &Path,
) -> Result<Vec<InstalledGuard>> {
    // Generate a container image tarball from `ContainerSettings`.
    let container_image_path = tempfile::Builder::new().suffix(".tar.gz").tempfile()?;
    let container_image_path = container_image_path.path();
    {
        let container = settings.prepare()?;
        let status = Command::new(locate_system_binary("tar")?)
            .arg("-czf")
            .arg(container_image_path)
            .arg("--exclude=dev")
            .arg("--exclude=sys")
            .arg("-C")
            .arg(container.root_dir())
            .arg(".")
            .status()?;
        ensure!(status.success(), "tar failed: {:?}", status);
    }

    // Generate contents directories and prepare output directories.
    let prefix = root_dir.strip_prefix("/")?.to_str().unwrap();
    let host = prefix.is_empty();
    let mut guards: Vec<InstalledGuard> = Vec::new();
    for binary_package in binary_packages {
        let installed_contents_dir = run_extract_package(binary_package, prefix, prefix, host)?;
        let staged_contents_dir = run_extract_package(binary_package, ".image", prefix, host)?;
        let preinst_dir = temp_dir_for_testing()?;
        let postinst_dir = temp_dir_for_testing()?;
        guards.push(InstalledGuard {
            installed_contents_dir,
            staged_contents_dir,
            preinst_dir,
            postinst_dir,
        });
    }

    let mut specs: Vec<InstallSpec> = Vec::new();
    for (binary_package, guard) in binary_packages.iter().zip(guards.iter()) {
        specs.push(InstallSpec {
            input_binary_package: binary_package,
            input_installed_contents_dir: guard.installed_contents_dir.path(),
            input_staged_contents_dir: guard.staged_contents_dir.path(),
            output_preinst_dir: guard.preinst_dir.path(),
            output_postinst_dir: guard.postinst_dir.path(),
        });
    }

    run_fast_install_packages(
        container_image_path,
        root_dir,
        &specs.iter().collect::<Vec<_>>(),
    )?;

    for spec in specs {
        settings.push_layer(spec.output_preinst_dir)?;
        settings.push_layer(spec.input_installed_contents_dir)?;
        settings.push_layer(spec.output_postinst_dir)?;
    }

    Ok(guards)
}

/// Creates a fake SDK container containing a minimal set of files, e.g.
/// drive_binary_package.sh, so that fast_install_packages works.
fn create_fake_sdk_container() -> Result<ContainerSettings> {
    let runfiles = Runfiles::create()?;
    let mut settings = ContainerSettings::new();
    settings.push_layer(
        &runfiles.rlocation("alpine-minirootfs/file/alpine-minirootfs-3.18.3-x86_64.tar.gz"),
    )?;
    settings.push_layer(
        &runfiles.rlocation("cros/bazel/portage/bin/fast_install_packages/fake_sdk_extras.tar"),
    )?;
    Ok(settings)
}

/// Creates a binary package for testing.
fn create_binary_package(
    cpf: &str,
    contents_dir: &Path,
    extra_environment: &str,
) -> Result<NamedTempFile> {
    let runfiles = Runfiles::create()?;
    let zstd_path = runfiles.rlocation("zstd/zstd");

    let binary_package_file = tempfile::Builder::new().suffix(".tbz2").tempfile()?;

    let status = Command::new(locate_system_binary("tar")?)
        .arg("-c")
        .arg("-f")
        .arg(binary_package_file.path())
        .arg("-I")
        .arg(&zstd_path)
        .arg("-C")
        .arg(contents_dir)
        .arg(".")
        .status()?;
    ensure!(status.success(), "tar failed: {:?}", status);

    // Compute XPAK key/values.
    let (category, pf) = cpf.split_once('/').expect("Invalid CPF");
    let environment = format!(
        "EAPI=7\nCATEGORY={}\nPF={}\nSLOT={}\n{}",
        category, pf, DEFAULT_SLOT, extra_environment
    );
    let mut environment_bz2: Vec<u8> = Vec::new();
    BzEncoder::new(environment.as_bytes(), Compression::fast())
        .read_to_end(&mut environment_bz2)?;

    let xpak_dict = [
        ("repository", Vec::from("chromiumos")),
        ("CATEGORY", Vec::from(category)),
        ("PF", Vec::from(pf)),
        ("SLOT", DEFAULT_SLOT.into()),
        ("environment.bz2", environment_bz2),
    ];

    // Serialize XPAK to bytes.
    // See https://www.mankier.com/5/xpak for the format specification.
    let mut index_section: Vec<u8> = Vec::new();
    let mut data_section: Vec<u8> = Vec::new();
    for (name, data) in xpak_dict {
        index_section.put_u32_be(name.len() as u32);
        index_section.extend(name.as_bytes());
        index_section.put_u32_be(data_section.len() as u32);
        index_section.put_u32_be(data.len() as u32);
        data_section.extend(data);
    }

    let mut xpak: Vec<u8> = Vec::new();
    xpak.extend_from_slice("XPAKPACK".as_bytes());
    xpak.put_u32_be(index_section.len() as u32);
    xpak.put_u32_be(data_section.len() as u32);
    xpak.extend(index_section);
    xpak.extend(data_section);
    xpak.extend_from_slice("XPAKSTOP".as_bytes());
    xpak.put_u32_be(xpak.len() as u32);
    xpak.extend_from_slice("STOP".as_bytes());

    // Append XPAK to the tarball.
    let mut w = OpenOptions::new()
        .append(true)
        .open(binary_package_file.path())?;
    w.write_all(&xpak)?;

    Ok(binary_package_file)
}

/// Tries installing zero packages.
#[test]
fn test_install_nothing() -> Result<()> {
    let mut settings = create_fake_sdk_container()?;
    fast_install_packages(&mut settings, &[], Path::new("/build/eve"))?;
    Ok(())
}

/// Tries installing a simple package without hooks.
#[test]
fn test_install_simple() -> Result<()> {
    let mut settings = create_fake_sdk_container()?;

    // Create a binary package that contains /usr/bin/ok.
    let contents_dir = TempDir::new()?;
    let contents_dir = contents_dir.path();
    std::fs::create_dir_all(contents_dir.join("usr/bin"))?;
    std::fs::write(contents_dir.join("usr/bin/ok"), "#!/bin/bash\nexit 0\n")?;
    std::fs::set_permissions(
        contents_dir.join("usr/bin/ok"),
        Permissions::from_mode(0o755),
    )?;

    let binary_package = create_binary_package("sys-apps/ok-1.0", contents_dir, "")?;

    let _guards = fast_install_packages(
        &mut settings,
        &[binary_package.path()],
        Path::new("/build/eve"),
    )?;

    // Try running /build/eve/usr/bin/ok in the container to verify that it's installed.
    let mut container = settings.prepare()?;
    let status = container.command("/build/eve/usr/bin/ok").status()?;
    assert!(
        status.success(),
        "/build/eve/usr/bin/ok failed: {:?}",
        status
    );

    // Inspect CONTENTS in VDB.
    let contents_path =
        get_vdb_dir(&container.root_dir().join("build/eve"), "sys-apps/ok-1.0").join("CONTENTS");
    let contents = std::fs::read_to_string(contents_path)?;
    assert_eq!(
        contents,
        r"dir usr
dir usr/bin
obj usr/bin/ok a3d4e05ae2b06745f36b715af15f1729 0
"
    );
    Ok(())
}

/// Tries installing a package containing no file (aka a virtual package).
#[test]
fn test_install_virtual() -> Result<()> {
    let mut settings = create_fake_sdk_container()?;

    // Create a binary package that contains no file.
    let contents_dir = TempDir::new()?;
    let contents_dir = contents_dir.path();
    let binary_package = create_binary_package("virtual/empty-1.0", contents_dir, "")?;

    let _guards = fast_install_packages(
        &mut settings,
        &[binary_package.path()],
        Path::new("/build/eve"),
    )?;

    // Inspect CONTENTS in VDB.
    let container = settings.prepare()?;
    let contents_path =
        get_vdb_dir(&container.root_dir().join("build/eve"), "virtual/empty-1.0").join("CONTENTS");
    let contents = std::fs::read_to_string(contents_path)?;
    assert_eq!(contents, r"");
    Ok(())
}

/// Tries installing a package with hooks.
#[test]
fn test_install_hooks() -> Result<()> {
    let mut settings = create_fake_sdk_container()?;

    // Create a binary package that contains /etc/foo.d/1.conf.
    let contents_dir = TempDir::new()?;
    let contents_dir = contents_dir.path();
    std::fs::create_dir_all(contents_dir.join("etc/foo.d"))?;
    std::fs::write(
        contents_dir.join("etc/foo.d/1.conf"),
        "this is a bad config\n",
    )?;

    // Define hooks that modify the file system.
    let extra_environment = r#"
    pkg_setup() {
        mkdir -p "${ROOT}/etc/foo.d"
        # Create 2.conf. This file will not be recorded in CONTENTS.
        echo "goodbye, world" > "${ROOT}/etc/foo.d/2.conf"
    }
    pkg_preinst() {
        # Rewrite 1.conf from the package contents.
        sed -i 's/bad/good/' "${D}/etc/foo.d/1.conf"
        # Rewrite 2.conf we created in pkg_setup.
        sed -i 's/goodbye/hello/' "${ROOT}/etc/foo.d/2.conf"
        # Create 3.conf. This file will be recorded in CONTENTS.
        echo "looks good to me" > "${D}/etc/foo.d/3.conf"
    }
    pkg_postinst() {
        # Merge all configs under /etc/foo.d/ to /etc/foo.conf.
        cat "${ROOT}"/etc/foo.d/*.conf > "${ROOT}/etc/foo.conf"
    }
    "#;

    let binary_package =
        create_binary_package("sys-apps/foo-1.0", contents_dir, extra_environment)?;

    let _guards = fast_install_packages(
        &mut settings,
        &[binary_package.path()],
        Path::new("/build/eve"),
    )?;

    // Verify the contents of /build/eve/etc/foo.conf.
    let container = settings.prepare()?;
    let conf = std::fs::read_to_string(container.root_dir().join("build/eve/etc/foo.conf"))?;
    assert_eq!(
        conf,
        "this is a good config\nhello, world\nlooks good to me\n"
    );

    // Inspect CONTENTS in VDB.
    let contents_path =
        get_vdb_dir(&container.root_dir().join("build/eve"), "sys-apps/foo-1.0").join("CONTENTS");
    let contents = std::fs::read_to_string(contents_path)?;
    assert_eq!(
        contents,
        r"dir etc
dir etc/foo.d
obj etc/foo.d/1.conf 8cf8b414c945a7607f156bbc5849e8cf 0
obj etc/foo.d/3.conf 3a812a6dcc0d5773cf315a66884a156c 0
"
    );
    Ok(())
}

/// Tries installing multiple packages at once.
#[test]
fn test_install_multiple() -> Result<()> {
    let mut settings = create_fake_sdk_container()?;

    let mut binary_packages: Vec<NamedTempFile> = Vec::new();

    for index in 0..10 {
        // Create a package that contains `/opt/files/{index}.txt` only.
        let contents_dir = TempDir::new()?;
        let contents_dir = contents_dir.path();
        std::fs::create_dir_all(contents_dir.join("opt/files"))?;
        std::fs::File::create(contents_dir.join(format!("opt/files/{index}.txt")))?;

        let mut extra_environment = format!("INDEX={index}\n");
        // Add pkg_setup to some packages because fast_package_install may
        // specially handle packages without hooks.
        if index % 2 == 0 {
            // Verify that the expected number of files are installed.
            extra_environment += r#"
            pkg_setup() {
                n=$(ls "${ROOT}/opt/files" | wc -l)
                if [[ "${n}" != "${INDEX}" ]]; then
                    die "${CATEGORY}/${PF}: pkg_setup: got ${n} files, want ${INDEX} files"
                fi
            }
            "#;
        }

        let binary_package = create_binary_package(
            &format!("sys-apps/pkg{index}"),
            contents_dir,
            &extra_environment,
        )?;
        binary_packages.push(binary_package);
    }

    // Install all packages at once.
    let _guards = fast_install_packages(
        &mut settings,
        &binary_packages.iter().map(|bp| bp.path()).collect_vec(),
        Path::new("/build/eve"),
    )?;

    let container = settings.prepare()?;

    // Inspect CONTENTS in VDB.
    for index in 0..10 {
        let contents_path = get_vdb_dir(
            &container.root_dir().join("build/eve"),
            &format!("sys-apps/pkg{index}"),
        )
        .join("CONTENTS");
        let contents = std::fs::read_to_string(contents_path)?;
        assert_eq!(
            contents,
            format!(
                r#"dir opt
dir opt/files
obj opt/files/{index}.txt d41d8cd98f00b204e9800998ecf8427e 0
"#
            )
        );
    }

    // Inspect installed files.
    for index in 0..10 {
        let path = container
            .root_dir()
            .join(format!("build/eve/opt/files/{index}.txt"));
        assert!(path.try_exists()?);
    }

    Ok(())
}
