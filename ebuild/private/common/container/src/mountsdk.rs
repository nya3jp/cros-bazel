// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::control::ControlChannel;
use crate::{BindMount, LoginMode};
use anyhow::{anyhow, ensure, Context, Result};
use fileutil::SafeTempDir;
use run_in_container_lib::RunInContainerConfig;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::instrument;

const SUDO_PATH: &str = "/usr/bin/sudo";

#[derive(Clone)]
pub struct MountSdkConfig {
    pub layer_paths: Vec<PathBuf>,
    pub bind_mounts: Vec<BindMount>,
    pub envs: BTreeMap<OsString, OsString>,

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
    config: RunInContainerConfig,

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
        let diff_dir = tmp_dir.path().join("diff");

        for dir in [&scratch_dir, &diff_dir] {
            std::fs::create_dir_all(dir)?;
        }

        let root_dir = fileutil::DualPath {
            outside: tmp_dir.path().join("root"),
            inside: PathBuf::from("/"),
        };
        let stage_dir = root_dir.join("mnt/host/.container");
        let control_channel_path = stage_dir.join("control");

        std::fs::create_dir_all(&stage_dir.outside)?;

        // Start with a clean environment.
        let mut envs: BTreeMap<OsString, OsString> = BTreeMap::new();
        let mut bind_mounts: Vec<BindMount> = cfg.bind_mounts;

        envs.extend([
            ("PATH".into(), "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/bin:/mnt/host/source/chromite/bin:/mnt/host/depot_tools".into()),
            // Always enable Rust backtrace.
            ("RUST_BACKTRACE".into(), "1".into()),
        ]);

        if let Some(board) = &board {
            envs.extend([("BOARD".into(), board.into())])
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
            envs.insert("_LOGIN_MODE".into(), cfg.login_mode.to_string().into());

            // Ensure we forward the TERM variable so bash behaves correctly.
            if let Some(term) = std::env::var_os("TERM") {
                envs.insert("_TERM".into(), term.into());
            }

            Some(ControlChannel::new(control_channel_path.outside)?)
        } else {
            None
        };

        envs.extend(cfg.envs);

        let cmd = if cfg.privileged {
            ensure_passwordless_sudo()?;
            let mut cmd = Command::new(SUDO_PATH);
            // We have no idea why, but run_in_container fails on pivot_root(2)
            // for EINVAL if we don't enter a mount namespace in advance.
            // TODO: Investigate the cause.
            cmd.args(["unshare", "--mount", "--"]);
            cmd.arg(run_in_container_path);
            cmd
        } else {
            Command::new(run_in_container_path)
        };

        let mut layer_paths: Vec<PathBuf> = cfg.layer_paths;
        layer_paths.push(root_dir.outside.clone());

        let setup_script_path = stage_dir.join("setup.sh");
        std::fs::copy(
            r.rlocation("cros/bazel/ebuild/private/common/container/setup.sh"),
            setup_script_path.outside,
        )?;

        let config = RunInContainerConfig {
            upper_dir: diff_dir.clone(),
            scratch_dir,
            args: vec![setup_script_path.inside.as_os_str().to_owned()], // appended later
            envs: envs.into_iter().collect(),
            chdir: PathBuf::from("/"),
            layer_paths,
            bind_mounts: bind_mounts.into_iter().map(|bm| bm.into_config()).collect(),
            allow_network_access: cfg.allow_network_access,
            privileged: cfg.privileged,
            keep_host_mount: false,
            resolve_symlink_forests: true,
        };

        Ok(Self {
            cmd: Some(cmd),
            root_dir,
            diff_dir,
            config,
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

        self.config
            .args
            .extend(args.into_iter().map(OsString::from));

        let config_path = self.tmp_dir.path().join("run_in_container_args.json");
        self.config.serialize_to(&config_path)?;

        cmd.arg("--config").arg(&config_path);

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
