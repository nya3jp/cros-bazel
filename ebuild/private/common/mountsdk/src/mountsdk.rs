// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::control::ControlChannel;
use anyhow::{anyhow, Context, Result};
use makechroot::{BindMount, OverlayInfo};
use run_in_container_lib::RunInContainerConfig;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use strum_macros::EnumString;
use tempfile::{tempdir, TempDir};

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
    pub overlays: Vec<OverlayInfo>,
    pub bind_mounts: Vec<BindMount>,
    pub envs: HashMap<String, String>,

    pub cmd_prefix: Vec<String>,
    pub(crate) login_mode: LoginMode,
    pub(crate) log_file: Option<PathBuf>,
}

pub struct MountedSDK {
    pub board: String,
    root_dir: fileutil::DualPath,
    diff_dir: PathBuf,
    log_file: Option<PathBuf>,

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
            r.rlocation("cros/bazel/ebuild/private/cmd/run_in_container/run_in_container_rust");

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

        let mut overlays: Vec<OverlayInfo> = vec![OverlayInfo {
            mount_dir: "/".into(),
            image_path: root_dir.outside.clone(),
        }];
        let mut bind_mounts: Vec<BindMount> = cfg.bind_mounts;
        let mut cmd = if cfg.cmd_prefix.is_empty() {
            Command::new(run_in_container_path)
        } else {
            let mut cmd = Command::new(&cfg.cmd_prefix[0]);
            cmd.args(&cfg.cmd_prefix[1..]).arg(run_in_container_path);
            cmd
        };
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let serialized_path = tmp_dir.path().join("run_in_container_args.json");
        cmd.arg("--cfg").arg(&serialized_path);

        overlays.extend(cfg.overlays);

        let setup_script_path = bazel_build_dir.join("setup.sh");
        std::fs::copy(
            r.rlocation("cros/bazel/ebuild/private/common/mountsdk/setup.sh"),
            setup_script_path.outside,
        )?;
        cmd.envs(std::env::vars());
        cmd.envs(cfg.envs);
        cmd.env("PATH", "/usr/sbin:/usr/bin:/sbin:/bin");
        cmd.env("BOARD", &cfg.board);
        let control_channel = if cfg.login_mode != LoginMode::Never {
            // Named pipes created using `mkfifo` use the inode number as the address.
            // We need to bind mount the control fifo on top of the overlayfs mounts to
            // prevent overlayfs from interfering with the device/inode lookup.
            bind_mounts.push(BindMount {
                mount_path: control_channel_path.inside,
                source: control_channel_path.outside.clone(),
            });
            cmd.env("_LOGIN_MODE", cfg.login_mode.to_string());
            Some(ControlChannel::new(control_channel_path.outside)?)
        } else {
            None
        };

        cmd.arg("--cmd").arg(setup_script_path.inside);
        RunInContainerConfig {
            staging_dir: scratch_dir,
            chdir: PathBuf::from("/"),
            overlays,
            bind_mounts,
            keep_host_mount: false,
        }
        .serialize_to(&serialized_path)?;
        return Ok(Self {
            board: cfg.board,
            cmd: Some(cmd),
            root_dir,
            diff_dir,
            log_file: cfg.log_file,
            tmp_dir,
            _control_channel: control_channel,
        });
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> Result<()> {
        let path = &self.root_dir.outside.join(path.as_ref().strip_prefix("/")?);
        std::fs::create_dir_all(path.parent().ok_or(anyhow!("Path can't be empty"))?)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn root_dir(&self) -> &fileutil::DualPath {
        &self.root_dir
    }

    pub fn diff_dir(&self) -> &PathBuf {
        &self.diff_dir
    }

    pub fn run_cmd(&mut self, args: &[&str]) -> Result<Output> {
        let mut cmd = self
            .cmd
            .take()
            .with_context(|| "Can only execute a command once per SDK.")?;
        cmd.args(args);
        if let Some(log_file) = &self.log_file {
            // Only redirect stderr if we aren't in an interactive shell (b/267392458).
            if !nix::unistd::isatty(0)? {
                return processes::run_suppress_stderr(&mut cmd, &log_file);
            }
        }
        processes::run_and_check(&mut cmd)
    }
}

impl Drop for MountedSDK {
    fn drop(&mut self) {
        fileutil::remove_dir_all_with_chmod(self.tmp_dir.path()).unwrap()
    }
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
            overlays: vec![
                OverlayInfo {
                    image_path: r.rlocation("cros/bazel/sdk/base_sdk"),
                    mount_dir: "/".into(),
                },
                OverlayInfo {
                    image_path: r.rlocation("cros/bazel/sdk/base_sdk-symlinks.tar"),
                    mount_dir: "/".into(),
                },
            ],
            bind_mounts: vec![
                BindMount {
                    source: r.rlocation(
                        "cros/bazel/ebuild/private/common/mountsdk/testdata/mypkg.ebuild",
                    ),
                    mount_path: ebuild_file.to_path_buf(),
                },
                BindMount {
                    source: hello.to_path_buf(),
                    mount_path: "/hello".into(),
                },
            ],
            envs: HashMap::new(),
            cmd_prefix: vec![],
            login_mode: Never,
            log_file: None,
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
