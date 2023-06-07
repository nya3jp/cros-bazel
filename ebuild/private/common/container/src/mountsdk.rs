// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::control::ControlChannel;
use crate::{BindMount, LoginMode};
use anyhow::{anyhow, ensure, Context, Result};
use fileutil::SafeTempDir;
use run_in_container_lib::RunInContainerConfig;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::instrument;

const SUDO_PATH: &str = "/usr/bin/sudo";

#[derive(Clone)]
pub struct MountSdkConfig {
    pub layer_paths: Vec<PathBuf>,
    pub bind_mounts: Vec<BindMount>,
    pub envs: HashMap<String, String>,

    /// If set to true, allows network access. This flag should be used only
    /// when it's absolutely needed since it reduces hermeticity.
    pub allow_network_access: bool,

    /// If set to true, runs a privileged container.
    pub privileged: bool,

    pub(crate) login_mode: LoginMode,
}

pub struct MountedSDK {
    root_dir: fileutil::DualPath,
    diff_dir: PathBuf,

    // Required for RAII.
    cmd: Option<Command>,
    _control_channel: Option<ControlChannel>,

    // pub(crate) required for testing.
    #[allow(dead_code)]
    pub(crate) tmp_dir: SafeTempDir,
}

impl MountedSDK {
    // Prepares the SDK according to the specifications requested.
    #[instrument(skip_all)]
    pub fn new(cfg: MountSdkConfig, board: Option<&str>) -> Result<Self> {
        let r = runfiles::Runfiles::create()?;
        let run_in_container_path =
            r.rlocation("cros/bazel/ebuild/private/cmd/run_in_container/run_in_container");

        let tmp_dir = SafeTempDir::new()?;

        let scratch_dir = tmp_dir.path().join("scratch");
        let diff_dir = scratch_dir.join("diff");
        let root_dir = fileutil::DualPath {
            outside: tmp_dir.path().join("root"),
            inside: PathBuf::from("/"),
        };
        let bazel_build_dir = root_dir.join("mnt/host/bazel-build");
        let control_channel_path = bazel_build_dir.join("control");

        std::fs::create_dir_all(&bazel_build_dir.outside)?;

        // Start with a clean environment.
        let mut envs: HashMap<String, String> = HashMap::new();
        let mut bind_mounts: Vec<BindMount> = cfg.bind_mounts;

        envs.extend([
            ("PATH".to_owned(), "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/bin:/mnt/host/source/chromite/bin:/mnt/host/depot_tools".to_owned()),
            // Always enable Rust backtrace.
            ("RUST_BACKTRACE".to_owned(), "1".to_owned()),
        ]);

        if let Some(board) = &board {
            envs.extend([("BOARD".to_owned(), board.to_string())])
        }

        let control_channel = if cfg.login_mode != LoginMode::Never {
            // Named pipes created using `mkfifo` use the inode number as the address.
            // We need to bind mount the control fifo on top of the overlayfs mounts to
            // prevent overlayfs from interfering with the device/inode lookup.
            bind_mounts.push(BindMount {
                mount_path: control_channel_path.inside,
                source: control_channel_path.outside.clone(),
                rw: false,
            });
            envs.insert("_LOGIN_MODE".to_owned(), cfg.login_mode.to_string());

            // Ensure we forward the TERM variable so bash behaves correctly.
            if let Some(term) = std::env::var_os("TERM") {
                // TODO: Switch envs over to store OsStrings
                envs.insert("_TERM".to_owned(), term.to_string_lossy().to_string());
            }

            Some(ControlChannel::new(control_channel_path.outside)?)
        } else {
            None
        };

        envs.extend(cfg.envs);

        let mut cmd = if cfg.privileged {
            ensure_passwordless_sudo()?;
            let mut cmd = Command::new(SUDO_PATH);
            // We have no idea why, but run_in_container fails on pivot_root(2)
            // for EINVAL if we don't enter a mount namespace in advance.
            // TODO: Investigate the cause.
            cmd.args(["unshare", "--mount", "--"]);
            cmd.arg(run_in_container_path);
            cmd.arg("--privileged");
            cmd
        } else {
            Command::new(run_in_container_path)
        };

        let mut layer_paths: Vec<PathBuf> = cfg.layer_paths;
        layer_paths.push(root_dir.outside.clone());
        let serialized_config = RunInContainerConfig {
            staging_dir: scratch_dir,
            envs: envs.into_iter().collect(),
            chdir: PathBuf::from("/"),
            layer_paths,
            bind_mounts: bind_mounts.into_iter().map(|bm| bm.into_config()).collect(),
            keep_host_mount: false,
        };
        let serialized_path = tmp_dir.path().join("run_in_container_args.json");
        serialized_config.serialize_to(&serialized_path)?;
        cmd.arg("--cfg").arg(&serialized_path);

        if cfg.allow_network_access {
            cmd.arg("--allow-network-access");
        }

        let setup_script_path = bazel_build_dir.join("setup.sh");
        std::fs::copy(
            r.rlocation("cros/bazel/ebuild/private/common/container/setup.sh"),
            setup_script_path.outside,
        )?;
        cmd.arg("--cmd").arg(setup_script_path.inside);

        Ok(Self {
            cmd: Some(cmd),
            root_dir,
            diff_dir,
            tmp_dir,
            _control_channel: control_channel,
        })
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> Result<()> {
        let path = &self.root_dir.outside.join(path.as_ref().strip_prefix("/")?);
        std::fs::create_dir_all(
            path.parent()
                .ok_or_else(|| anyhow!("Path can't be empty"))?,
        )?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn root_dir(&self) -> &fileutil::DualPath {
        &self.root_dir
    }

    pub fn diff_dir(&self) -> &PathBuf {
        &self.diff_dir
    }

    #[instrument(skip_all)]
    pub fn run_cmd(&mut self, args: &[&str]) -> Result<()> {
        let mut cmd = self
            .cmd
            .take()
            .with_context(|| "Can only execute a command once per SDK.")?;
        cmd.args(args);
        processes::run_and_check(&mut cmd)
    }
}

fn ensure_passwordless_sudo() -> Result<()> {
    let status = Command::new(SUDO_PATH)
        .args(["-n", "true"])
        .status()
        .context("Failed to run sudo")?;
    ensure!(
        status.success(),
        "Failed to run sudo without password; run \"sudo true\" and try again"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use runfiles::Runfiles;

    const SOURCE_DIR: &str = "/mnt/host/source";

    #[test]
    fn run_in_sdk() -> Result<()> {
        let r = Runfiles::create()?;
        let tmp_dir = SafeTempDir::new()?;
        let hello = tmp_dir.path().join("hello");
        std::fs::write(&hello, "hello")?;

        let src_dir = PathBuf::from(SOURCE_DIR);
        let portage_stable = src_dir.join("src/third_party/portage-stable");
        let ebuild_file = portage_stable.join("mypkg/mypkg.ebuild");
        let board = Some("amd64-generic");
        let cfg = MountSdkConfig {
            layer_paths: vec![r.rlocation("cros/bazel/sdk/sdk_from_archive")],
            bind_mounts: vec![
                BindMount {
                    source: r.rlocation(
                        "cros/bazel/ebuild/private/common/container/testdata/mypkg.ebuild",
                    ),
                    mount_path: ebuild_file.clone(),
                    rw: false,
                },
                BindMount {
                    source: hello.clone(),
                    mount_path: "/hello".into(),
                    rw: false,
                },
            ],
            envs: HashMap::new(),
            allow_network_access: false,
            privileged: false,
            login_mode: LoginMode::Never,
        };

        MountedSDK::new(cfg.clone(), board)?.run_cmd(&["true"])?;

        assert!(MountedSDK::new(cfg.clone(), board)?
            .run_cmd(&["false"])
            .is_err());

        // Should be a read-only mount.
        assert!(MountedSDK::new(cfg.clone(), board)?
            .run_cmd(&["/bin/bash", "-c", "echo world > /hello"])
            .is_err());
        // Verify that the chroot hasn't modified this file.
        assert_eq!(std::fs::read_to_string(hello)?, "hello");

        // Check we're in the SDK by using a binary unlikely to be on the host machine.
        MountedSDK::new(cfg.clone(), board)?.run_cmd(&["test", "-f", "/usr/bin/ebuild"])?;

        MountedSDK::new(cfg.clone(), board)?.run_cmd(&[
            "grep",
            "EBUILD_CONTENTS",
            &ebuild_file.to_string_lossy(),
        ])?;

        let tmp_dir: PathBuf = {
            let mut sdk = MountedSDK::new(cfg, board)?;

            let out_pkg = sdk.root_dir.join("build/arm64-generic/packages/mypkg");
            std::fs::create_dir_all(&out_pkg.outside)?;

            let out_file = out_pkg.inside.join("mypkg.tbz2");
            sdk.run_cmd(&["touch", &out_file.to_string_lossy()])?;
            assert!(sdk
                .diff_dir
                .join(out_file.strip_prefix("/")?)
                .try_exists()?);

            sdk.tmp_dir.path().to_path_buf()
        };
        // Now that the SDK has gone out of scope, it should clean up the directory.
        assert!(!tmp_dir.try_exists()?);

        Ok(())
    }
}
