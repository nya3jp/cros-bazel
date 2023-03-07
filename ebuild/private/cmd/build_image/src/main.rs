// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use clap::Parser;
use makechroot::BindMount;
use mountsdk::{InstallGroup, MountedSDK};
use std::path::{Path, PathBuf};
use std::str::from_utf8;

const MAIN_SCRIPT: &str = "/mnt/host/bazel-build/build_image.sh";

#[derive(Parser, Debug)]
#[clap()]
pub struct Cli {
    #[command(flatten)]
    mountsdk_config: mountsdk::ConfigArgs,

    #[arg(long)]
    output: PathBuf,

    #[arg(long, required = true)]
    install_target: Vec<InstallGroup>,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let r = runfiles::Runfiles::create()?;

    let mut cfg = mountsdk::Config::try_from(args.mountsdk_config)?;
    cfg.privileged = true;

    cfg.bind_mounts.push(BindMount {
        source: r
            .rlocation("cros/bazel/ebuild/private/cmd/build_image/container_files/edb_chromeos"),
        mount_path: Path::new("/build")
            .join(&cfg.board)
            .join("var/cache/edb/chromeos"),
    });
    cfg.bind_mounts.push(BindMount {
        source: r.rlocation(
            "cros/bazel/ebuild/private/cmd/build_image/container_files/package.provided",
        ),
        mount_path: Path::new("/build")
            .join(&cfg.board)
            .join("etc/portage/profile/package.provided"),
    });
    cfg.bind_mounts.push(BindMount {
        source: r.rlocation("cros/bazel/ebuild/private/cmd/install_deps/install_deps.sh"),
        mount_path: PathBuf::from("/mnt/host/bazel-build/install_deps.sh"),
    });
    cfg.bind_mounts.push(BindMount {
        source: r
            .rlocation("cros/bazel/ebuild/private/cmd/build_image/container_files/build_image.sh"),
        mount_path: PathBuf::from(MAIN_SCRIPT),
    });

    // base_image_util calls install_libc_for_abi, which expects certain
    // cross-compilation tools to be stored at specific locations.
    // TODO: Once we can build with custom use flags, stop hardcoding aarch64.

    // It's possible not all of these packages are needed. I may remove some
    // later if we find out they're never needed throughout the whole
    // build_image process.
    for resource in [
        "amd64_host_cross_aarch64_cros_linux_gnu_binutils_2_36_1_r8/file/binutils-2.36.1-r8.tbz2",
        "amd64_host_cross_aarch64_cros_linux_gnu_compiler_rt_15_0_pre458507_r6/file/compiler-rt-15.0_pre458507-r6.tbz2",
        "amd64_host_cross_aarch64_cros_linux_gnu_gcc_10_2_0_r28/file/gcc-10.2.0-r28.tbz2",
        "amd64_host_cross_aarch64_cros_linux_gnu_gdb_9_2_20200923_r9/file/gdb-9.2.20200923-r9.tbz2",
        "amd64_host_cross_aarch64_cros_linux_gnu_glibc_2_33_r17/file/glibc-2.33-r17.tbz2",
        "amd64_host_cross_aarch64_cros_linux_gnu_go_1_18_r2/file/go-1.18-r2.tbz2",
        "amd64_host_cross_aarch64_cros_linux_gnu_libcxx_15_0_pre458507_r6/file/libcxx-15.0_pre458507-r6.tbz2",
        "amd64_host_cross_aarch64_cros_linux_gnu_libxcrypt_4_4_28_r1/file/libxcrypt-4.4.28-r1.tbz2",
        "amd64_host_cross_aarch64_cros_linux_gnu_linux_headers_4_14_r56/file/linux-headers-4.14-r56.tbz2",
        "amd64_host_cross_aarch64_cros_linux_gnu_llvm_libunwind_15_0_pre458507_r4/file/llvm-libunwind-15.0_pre458507-r4.tbz2",
    ] {
        // TODO: install_libc hardcodes arm64 to also install the arm32 packages.
        // This is required only if nacl is used.
        // For now, install_libc succeeds if we comment out this hardcoding.
        // Once we can build with custom use flags, we can then support this
        // properly.
        // https://source.chromium.org/chromiumos/chromiumos/codesearch/+/main:src/scripts/build_library/base_image_util.sh;l=272-278;drc=cdaf1eab71d4e607239ccc9db877ff2a22f8568e
        let source = r.rlocation(resource);
        cfg.bind_mounts.push(BindMount {
            mount_path: Path::new("/var/lib/portage/pkgs/cross-aarch64-cros-linux-gnu").join(
                source
                    .file_name()
                    .with_context(|| "Resource must have a filename")?,
            ),
            source,
        });
    }

    let target_packages_dir = Path::new("/build").join(&cfg.board).join("packages");
    // get_mounts_and_env returns a set of environment variables for the packages you want to
    // install. We want to drop this to avoid calling emerge on every package which we know is
    // already installed.
    let (mut mounts, env) =
        InstallGroup::get_mounts_and_env(&args.install_target, target_packages_dir)?;
    cfg.bind_mounts.append(&mut mounts);
    // setup_board.sh creates emerge-{board} and portageq-{board}, both of
    // which are used by build_image.sh
    let board_script_template = &std::fs::read(
        r.rlocation("cros/bazel/ebuild/private/cmd/build_image/container_files/board_script.sh"),
    )?;
    // TODO: stop hardcoding aarch64-cros-linux-gnu.
    let board_script = from_utf8(board_script_template)?
        .replace("${BOARD}", &cfg.board)
        .replace("${CHOST}", "aarch64-cros-linux-gnu");

    cfg.envs = env;
    cfg.envs
        .insert("HOST_UID".to_owned(), users::get_current_uid().to_string());
    cfg.envs
        .insert("HOST_GID".to_owned(), users::get_current_gid().to_string());

    let mut sdk = MountedSDK::new(cfg)?;
    sdk.write(
        format!("/usr/bin/emerge-{}", &sdk.board),
        board_script.replace("${COMMAND}", "emerge --root-deps"),
    )?;
    sdk.write(
        format!("/usr/bin/portageq-{}", &sdk.board),
        board_script.replace("${COMMAND}", "portageq"),
    )?;

    sdk.run_cmd(&[
        MAIN_SCRIPT,
        &format!("--board={}", &sdk.board),
        // TODO: at some point, we should support a variety of image types
        "base",
        // TODO: add unparsed command-line args.
    ])?;

    let path = Path::new("mnt/host/source/src/build/images")
        .join(&sdk.board)
        .join("latest_chromiumos_base_image.bin");
    std::fs::copy(sdk.diff_dir().join(path), args.output)?;

    Ok(())
}
