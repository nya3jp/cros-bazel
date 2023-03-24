// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::control::ControlChannel;
use anyhow::{anyhow, bail, ensure, Context, Result};
use itertools::Itertools;
use makechroot::BindMount;
use run_in_container_lib::RunInContainerConfig;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::Command;
use strum_macros::EnumString;
use tempfile::{tempdir, TempDir};

const SUDO_PATH: &str = "/usr/bin/sudo";

#[derive(Debug, Clone, Copy, PartialEq, EnumString, strum_macros::Display)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum LoginMode {
    #[strum(serialize = "")]
    Never,
    Before,
    After,
    AfterFail,
}

#[derive(Clone)]
pub struct Config {
    pub board: String,
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
    pub board: String,
    root_dir: fileutil::DualPath,
    diff_dir: PathBuf,
    privileged: bool,

    // Required for RAII.
    cmd: Option<Command>,
    _control_channel: Option<ControlChannel>,
    // pub(crate) required for testing.
    pub(crate) tmp_dir: TempDir,
}

impl MountedSDK {
    // Prepares the SDK according to the specifications requested.
    pub fn new(cfg: Config) -> Result<Self> {
        let r = runfiles::Runfiles::create()?;
        let run_in_container_path =
            r.rlocation("cros/bazel/ebuild/private/cmd/run_in_container/run_in_container");

        let tmp_dir = tempdir()?;

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
            // TODO: Do not set $BOARD here. Callers may not be interested in
            // particular boards.
            ("BOARD".to_owned(), cfg.board.clone()),
        ]);

        let control_channel = if cfg.login_mode != LoginMode::Never {
            // Named pipes created using `mkfifo` use the inode number as the address.
            // We need to bind mount the control fifo on top of the overlayfs mounts to
            // prevent overlayfs from interfering with the device/inode lookup.
            bind_mounts.push(BindMount {
                mount_path: control_channel_path.inside,
                source: control_channel_path.outside.clone(),
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
            cmd.args(["unshare", "--mount", "--", "/usr/bin/env", "-i"]);
            cmd.args(envs.into_iter().sorted().map(|(k, v)| format!("{k}={v}")));
            cmd.arg(run_in_container_path);
            cmd.arg("--privileged");
            cmd.env_clear();
            cmd
        } else {
            let mut cmd = Command::new(run_in_container_path);
            cmd.env_clear();
            cmd.envs(envs);
            cmd
        };

        let layer_paths: Vec<PathBuf> = cfg.layer_paths.into_iter().map(
                |layer|
                // Convert the path to an absolute path if it's a runfile path prefixed with
                // "%runfiles/".
                // TODO(b/269558613): Fix all call sites to always use runfile paths and delete this
                // hack.
                if let Ok(path) = layer.strip_prefix("%runfiles") {
                    r.rlocation(path)
                } else {
                    layer
                }
            ).chain([root_dir.outside.clone()])
            .collect();
        let serialized_config = RunInContainerConfig {
            staging_dir: scratch_dir,
            chdir: PathBuf::from("/"),
            layer_paths,
            bind_mounts,
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
            r.rlocation("cros/bazel/ebuild/private/common/mountsdk/setup.sh"),
            setup_script_path.outside,
        )?;
        cmd.arg("--cmd").arg(setup_script_path.inside);

        Ok(Self {
            board: cfg.board,
            cmd: Some(cmd),
            root_dir,
            diff_dir,
            privileged: cfg.privileged,
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

    pub fn run_cmd(&mut self, args: &[&str]) -> Result<()> {
        let mut cmd = self
            .cmd
            .take()
            .with_context(|| "Can only execute a command once per SDK.")?;
        let status = cmd.args(args).status()?;
        if !status.success() {
            bail!("command failed: {:?}", status);
        }
        Ok(())
    }
}

impl Drop for MountedSDK {
    fn drop(&mut self) {
        if self.privileged {
            fileutil::remove_dir_all_with_sudo(self.tmp_dir.path()).unwrap()
        } else {
            fileutil::remove_dir_all_with_chmod(self.tmp_dir.path()).unwrap()
        }
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
    use crate::LoginMode::Never;
    use crate::SOURCE_DIR;
    use runfiles::Runfiles;

    #[test]
    fn run_in_sdk() -> Result<()> {
        let r = Runfiles::create()?;
        let tmp_dir = tempfile::tempdir()?;
        let hello = tmp_dir.path().join("hello");
        std::fs::write(&hello, "hello")?;

        let src_dir = PathBuf::from(SOURCE_DIR);
        let portage_stable = src_dir.join("src/third_party/portage-stable");
        let ebuild_file = portage_stable.join("mypkg/mypkg.ebuild");
        let cfg = Config {
            board: "amd64-generic".to_owned(),
            layer_paths: vec![
                r.rlocation("cros/bazel/sdk/base_sdk"),
                r.rlocation("cros/bazel/sdk/base_sdk-symlinks.tar"),
            ],
            bind_mounts: vec![
                BindMount {
                    source: r.rlocation(
                        "cros/bazel/ebuild/private/common/mountsdk/testdata/mypkg.ebuild",
                    ),
                    mount_path: ebuild_file.clone(),
                },
                BindMount {
                    source: hello.clone(),
                    mount_path: "/hello".into(),
                },
            ],
            envs: HashMap::new(),
            allow_network_access: false,
            privileged: false,
            login_mode: Never,
        };

        MountedSDK::new(cfg.clone())?.run_cmd(&["true"])?;

        assert!(MountedSDK::new(cfg.clone())?.run_cmd(&["false"]).is_err());

        // Should be a read-only mount.
        assert!(MountedSDK::new(cfg.clone())?
            .run_cmd(&["/bin/bash", "-c", "echo world > /hello"])
            .is_err());
        // Verify that the chroot hasn't modified this file.
        assert_eq!(std::fs::read_to_string(hello)?, "hello");

        // Check we're in the SDK by using a binary unlikely to be on the host machine.
        MountedSDK::new(cfg.clone())?.run_cmd(&["test", "-f", "/usr/bin/ebuild"])?;

        MountedSDK::new(cfg.clone())?.run_cmd(&[
            "grep",
            "EBUILD_CONTENTS",
            &ebuild_file.to_string_lossy(),
        ])?;

        let tmp_dir: PathBuf = {
            let mut sdk = MountedSDK::new(cfg)?;

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
