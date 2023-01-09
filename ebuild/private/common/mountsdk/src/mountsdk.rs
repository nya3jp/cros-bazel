// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::control::ControlChannel;
use anyhow::{anyhow, bail, Result};
use makechroot::{BindMount, OverlayInfo};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use strum_macros::EnumString;
use tempfile::{tempdir, TempDir};

#[derive(Debug, Clone, Copy, PartialEq, EnumString, strum_macros::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum LoginMode {
    #[strum(serialize = "")]
    Never,
    Before,
    After,
    AfterFail,
}

pub struct Config {
    pub overlays: Vec<OverlayInfo>,
    pub bind_mounts: Vec<BindMount>,
    // A list of paths which need to be remounted on top of the overlays.
    // For example, if you specify an overlay for /dir, but you want /dir/subdir
    // to come from the host, add /dir/subdir to Remounts.
    pub remounts: Vec<PathBuf>,

    pub run_in_container_extra_args: Vec<String>,
    pub login_mode: LoginMode,
}

pub struct MountedSDK {
    root_dir: fileutil::DualPath,
    diff_dir: PathBuf,

    args: Vec<String>,
    env: HashMap<String, String>,

    // Required for RAII.
    _control_channel: Option<ControlChannel>,
    tmp_dir: TempDir,
}

impl MountedSDK {
    // Prepares the SDK according to the specifications requested.
    pub fn new(cfg: Config) -> Result<Self> {
        let r = runfiles::Runfiles::create()?;
        let run_in_container_path = r.rlocation(
            "chromiumos/bazel/ebuild/private/cmd/run_in_container/run_in_container_/run_in_container",
        );

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

        let mut args: Vec<String> = vec![
            run_in_container_path.to_string_lossy().to_string(),
            format!("--scratch-dir={}", scratch_dir.to_string_lossy()),
            format!("--overlay=/={}", root_dir.outside.to_string_lossy()),
        ];
        args.extend(cfg.run_in_container_extra_args);

        for remount in cfg.remounts.iter() {
            if !remount.is_absolute() {
                bail!(
                    "expected remounts to be an absolute path: got {:?}",
                    remount
                );
            }
            let dual_path = root_dir.join(remount.strip_prefix("/")?);
            std::fs::create_dir_all(&dual_path.outside)?;
            args.push(format!(
                "--overlay={}={}",
                dual_path.inside.to_string_lossy().to_string(),
                dual_path.outside.to_string_lossy().to_string()
            ));
        }

        for overlay in cfg.overlays {
            args.push(format!(
                "--overlay={}={}",
                overlay.mount_dir.to_string_lossy().to_string(),
                overlay.image_path.to_string_lossy().to_string()
            ))
        }

        for bind_mount in cfg.bind_mounts {
            args.push(format!(
                "--bind-mount={}={}",
                bind_mount.mount_path.to_string_lossy().to_string(),
                bind_mount.source.to_string_lossy().to_string()
            ))
        }

        if cfg.login_mode != LoginMode::Never {
            // Named pipes created using `mkfifo` use the inode number as the address.
            // We need to bind mount the control fifo on top of the overlayfs mounts to
            // prevent overlayfs from interfering with the device/inode lookup.
            args.push(format!(
                "--bind-mount={}={}",
                control_channel_path.inside.to_string_lossy().to_string(),
                control_channel_path.outside.to_string_lossy().to_string()
            ))
        }

        let setup_script_path = bazel_build_dir.join("setup.sh");
        std::fs::copy(
            r.rlocation("chromiumos/bazel/ebuild/private/common/mountsdk/setup.sh"),
            setup_script_path.outside,
        )?;
        args.push(setup_script_path.inside.to_string_lossy().to_string());
        let mut env: HashMap<String, String> = std::env::vars().collect();
        env.insert(
            "PATH".to_string(),
            "/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
        );
        let control_channel = if cfg.login_mode != LoginMode::Never {
            env.insert("_LOGIN_MODE".to_string(), cfg.login_mode.to_string());
            Some(ControlChannel::new(control_channel_path.outside)?)
        } else {
            None
        };
        return Ok(Self {
            args,
            env,
            root_dir,
            diff_dir,
            tmp_dir: tmp_dir,
            _control_channel: control_channel,
        });
    }

    pub fn base_command(&self) -> Command {
        let mut cmd = Command::new(&self.args[0]);
        cmd.args(&self.args[1..])
            .envs(self.env.clone())
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        return cmd;
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> Result<()> {
        let path = self
            .root_dir
            .outside
            .join(PathBuf::from(path.as_ref()).strip_prefix("/")?);
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
    use processes::run_and_check;
    use runfiles::Runfiles;

    #[test]
    fn run_in_sdk() -> Result<()> {
        let r = Runfiles::create()?;
        let tmp_dir = tempfile::tempdir()?;
        let hello = tmp_dir.path().join("hello");
        std::fs::write(&hello, "hello")?;
        // These values were obtained by looking at an invocation of build_package.
        let src_dir = PathBuf::from(SOURCE_DIR);
        let portage_stable = src_dir.join("src/third_party/portage-stable");
        let ebuild_file = portage_stable.join("mypkg/mypkg.ebuild");
        let cfg = Config {
            overlays: vec![
                OverlayInfo {
                    image_path: r.rlocation("chromiumos/bazel/sdk/sdk"),
                    mount_dir: "/".into(),
                },
                OverlayInfo {
                    image_path: r.rlocation("chromiumos/bazel/sdk/sdk.symindex"),
                    mount_dir: "/".into(),
                },
                OverlayInfo {
                    image_path: r.rlocation("chromiumos/bazel/sdk/base_sdk"),
                    mount_dir: "/".into(),
                },
                OverlayInfo {
                    image_path: r.rlocation("chromiumos/bazel/sdk/base_sdk.symindex"),
                    mount_dir: "/".into(),
                },
                OverlayInfo {
                    image_path: r.rlocation(
                        "chromiumos/overlays/overlay-arm64-generic/overlay-arm64-generic.squashfs",
                    ),
                    mount_dir: src_dir.join("src/overlays/overlay-arm64-generic"),
                },
                OverlayInfo {
                    image_path: r
                        .rlocation("chromiumos/third_party/eclass-overlay/eclass-overlay.squashfs"),
                    mount_dir: src_dir.join("src/third_party/eclass-overlay"),
                },
                OverlayInfo {
                    image_path: r.rlocation(
                        "chromiumos/third_party/chromiumos-overlay/chromiumos-overlay.squashfs",
                    ),
                    mount_dir: src_dir.join("src/third_party/chromiumos-overlay"),
                },
                OverlayInfo {
                    image_path: r
                        .rlocation("chromiumos/third_party/portage-stable/portage-stable.squashfs"),
                    mount_dir: portage_stable.to_path_buf(),
                },
            ],
            bind_mounts: vec![
                BindMount {
                    source: r.rlocation(
                        "chromiumos/bazel/ebuild/private/common/mountsdk/testdata/mypkg.ebuild",
                    ),
                    mount_path: ebuild_file.to_path_buf(),
                },
                BindMount {
                    source: hello.to_path_buf(),
                    mount_path: "/hello".into(),
                },
            ],
            remounts: vec![portage_stable.join("mypkg")],
            run_in_container_extra_args: vec![],
            login_mode: Never,
        };

        let tmp_dir: PathBuf = {
            let sdk = MountedSDK::new(cfg)?;
            run_and_check(sdk.base_command().arg("true"))?;
            assert!(run_and_check(sdk.base_command().arg("false")).is_err());

            // Should be a read-only mount.
            assert!(run_and_check(sdk.base_command().args([
                "/bin/bash",
                "-c",
                "echo world > /hello"
            ]))
            .is_err());
            // Verify that the chroot hasn't modified this file.
            assert_eq!(std::fs::read_to_string(hello)?, "hello");

            // Check we're in the SDK by using a binary unlikely to be on the host machine.
            run_and_check(sdk.base_command().args(["test", "-f", "/usr/bin/ebuild"]))?;
            // Confirm that overlays were loaded in to the SDK.
            run_and_check(sdk.base_command().args([
                "test",
                "-d",
                &portage_stable.join("eclass").to_string_lossy(),
            ]))?;

            let out_pkg = sdk.root_dir.join("build/arm64-generic/packages/mypkg");
            std::fs::create_dir_all(&out_pkg.outside)?;
            run_and_check(sdk.base_command().args([
                "test",
                "-d",
                &out_pkg.inside.to_string_lossy(),
            ]))?;

            run_and_check(
                sdk.base_command()
                    .args(["test", "-f", &ebuild_file.to_string_lossy()]),
            )?;
            run_and_check(sdk.base_command().args([
                "grep",
                "EBUILD_CONTENTS",
                &ebuild_file.to_string_lossy(),
            ]))?;

            let out_file = out_pkg.inside.join("mypkg.tbz2");
            run_and_check(
                sdk.base_command()
                    .args(["touch", &out_file.to_string_lossy()]),
            )?;
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
