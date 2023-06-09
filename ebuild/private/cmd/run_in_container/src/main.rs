// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{ensure, Context, Result};
use clap::Parser;
use cliutil::{cli_main, handle_top_level_result, print_current_command_line};
use fileutil::SafeTempDir;
use itertools::Itertools;
use nix::{
    errno::Errno,
    mount::MntFlags,
    mount::{mount, umount2, MsFlags},
    sched::{unshare, CloneFlags},
    sys::socket::{socket, AddressFamily, SockFlag, SockProtocol, SockType},
    unistd::{getgid, getuid, pivot_root},
};
use path_absolutize::Absolutize;
use processes::status_to_exit_code;
use run_in_container_lib::RunInContainerConfig;
use std::{
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::fs::{DirBuilderExt, OpenOptionsExt},
    },
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};
use tracing::info_span;

const BIND_REC: MsFlags = MsFlags::MS_BIND.union(MsFlags::MS_REC);
const NONE_STR: Option<&str> = None::<&str>;

#[derive(Parser, Debug)]
struct Cli {
    /// A path to a serialized RunInContainerConfig.
    #[arg(long, required = true)]
    config: PathBuf,

    /// Whether we are already in the namespace. Never set this, as it's as internal flag.
    #[arg(long)]
    already_in_namespace: bool,
}

pub fn main() -> ExitCode {
    let args = Cli::parse();

    if !args.already_in_namespace {
        print_current_command_line();
        let result = || -> Result<_> {
            enter_namespace(RunInContainerConfig::deserialize_from(&args.config)?)
        }();
        handle_top_level_result(result)
    } else {
        cli_main(|| continue_namespace(RunInContainerConfig::deserialize_from(&args.config)?))
    }
}

fn enter_namespace(cfg: RunInContainerConfig) -> Result<ExitCode> {
    let r = runfiles::Runfiles::create()?;
    let dumb_init_path = r.rlocation("files/dumb_init");

    // Enter a new user namespace if unprivileged.
    if !cfg.privileged {
        let uid = getuid();
        let gid = getgid();
        unshare(CloneFlags::CLONE_NEWUSER)
            .with_context(|| "Failed to create an unprivileged user namespace")?;
        std::fs::write("/proc/self/setgroups", "deny")
            .with_context(|| "Writing /proc/self/setgroups")?;
        std::fs::write("/proc/self/uid_map", format!("0 {uid} 1\n"))
            .with_context(|| "Writing /proc/self/uid_map")?;
        std::fs::write("/proc/self/gid_map", format!("0 {gid} 1\n"))
            .with_context(|| "Writing /proc/self/gid_map")?;
    }

    // Enter various namespaces except mount/PID namespace.
    let mut unshare_flags = CloneFlags::CLONE_NEWIPC;
    if !cfg.allow_network_access {
        unshare_flags |= CloneFlags::CLONE_NEWNET;
    }
    unshare(unshare_flags)
        .with_context(|| format!("Failed to enter namespaces (flags={:?})", unshare_flags))?;

    // HACK: Start a "sentinel" subprocess that belongs to the new namespaces
    // (except PID namespace) and exits *after* the current process.
    //
    // Some namespaces (e.g. network) are expensive to destroy, so it takes some
    // time for the last process in the namespaces to exit. If we do it simply,
    // the current process would be the last process, so its parent process
    // needs to wait for namespace cleanup.
    //
    // We work around this problem by starting a subprocess that remains in the
    // namespaces. Specifically, we start a cat process whose stdin is piped,
    // and leak the process intentionally. When the current process exits, the
    // kernel closes the writer end of the pipe, which causes the cat process
    // to exit.
    //
    // Note that we can't run the subprocess in the new PID namespace because,
    // once the current process calls unshare(CLONE_NEWPID), it is limited to
    // call fork at most once. But fortunately it seems like destroying a PID
    // namespace is cheap.
    let sentinel = Command::new("/bin/cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    std::mem::forget(sentinel);

    // Enter a PID namespace.
    unshare(CloneFlags::CLONE_NEWPID).context("Failed to enter PID namespace")?;

    // Create a temporary directory to be used by the child run_in_container.
    // Since it enters a new mount namespace and calls pivot_root, it cannot
    // delete temporary directories they create, so this process takes care of
    // them.
    let temp_dir = SafeTempDir::new()?;

    // --single-child tells dumb-init to not create a new SID. A new SID doesn't
    // have a controlling terminal, so running `bash` won't work correctly.
    // By omitting the new SID creation, the init processes will inherit the
    // current (outside) SID and PGID. This is desirable because then the parent
    // shell can correctly perform job control (Ctrl+Z) on all the processes.
    // It also tells dumb-init to only forward signals to the child, instead of
    // the child's PGID, this is undesirable, but not really a problem in
    // practice. The other processes we run are `squashfsfuse`, and these create
    // their own SID's, so we were never forwarding the signals to those processes
    // in the first place. Honestly, I'm not sure if we really even want signal
    // forwarding. Instead our `init` processes should only handle
    // `SIGINT`/`SIGTERM`, perform a `kill -TERM -1` to notify all the processes
    // in the PID namespace to shut down cleanly, then wait for all processes
    // to exit.
    let args: Vec<String> = std::env::args().collect();
    let status = processes::run(
        Command::new(dumb_init_path)
            .arg("--single-child")
            .arg(&args[0])
            .arg("--already-in-namespace")
            .args(&args[1..])
            .env("TMPDIR", temp_dir.path()),
    )?;

    // Propagate the exit status of the command.
    Ok(status_to_exit_code(&status))
}

/// Enables the loopback networking.
fn enable_loopback_networking() -> Result<()> {
    let socket = unsafe {
        OwnedFd::from_raw_fd(
            socket(
                AddressFamily::Inet,
                SockType::Datagram,
                SockFlag::SOCK_CLOEXEC,
                SockProtocol::Udp,
            )
            .context("socket(AF_INET, SOCK_DGRAM) failed")?,
        )
    };

    let mut ifreq = libc::ifreq {
        ifr_name: [
            // The loopback device "lo".
            'l' as i8, 'o' as i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        ifr_ifru: libc::__c_anonymous_ifr_ifru { ifru_flags: 0 },
    };

    // Query the current flags.
    let res = unsafe { libc::ioctl(socket.as_raw_fd(), libc::SIOCGIFFLAGS, &ifreq) };
    Errno::result(res).context("ioctl(SIOCGIFFLAGS) failed")?;

    // Update the flags.
    unsafe {
        ifreq.ifr_ifru.ifru_flags |= (libc::IFF_UP | libc::IFF_RUNNING) as libc::c_short;
    }
    let res = unsafe { libc::ioctl(socket.as_raw_fd(), libc::SIOCSIFFLAGS, &ifreq) };
    Errno::result(res).context("ioctl(SIOCSIFFLAGS) failed")?;

    Ok(())
}

fn continue_namespace(cfg: RunInContainerConfig) -> Result<ExitCode> {
    unshare(CloneFlags::CLONE_NEWNS).context("Failed to enter mount namespace")?;

    let upper_dir = cfg.upper_dir.absolutize()?.into_owned();
    let scratch_dir = cfg.scratch_dir.absolutize()?.into_owned();

    if !cfg.allow_network_access {
        enable_loopback_networking()?;
    }

    // We keep all the directories in the stage dir to keep relative file paths short.
    let root_dir = scratch_dir.join("root"); // Merged directory
    let base_dir = scratch_dir.join("base"); // Directory containing mount targets
    let lowers_dir = scratch_dir.join("lowers");
    let work_dir = scratch_dir.join("work");

    let mut binding = std::fs::DirBuilder::new();
    let dir_builder = binding.recursive(true).mode(0o755);
    for dir in [&root_dir, &base_dir, &lowers_dir, &work_dir] {
        dir_builder.create(dir)?;
    }

    for dir in [&base_dir, &lowers_dir] {
        // Mount a tmpfs so that files are purged automatically on exit.
        mount(
            Some("tmpfs"),
            dir,
            Some("tmpfs"),
            MsFlags::empty(),
            NONE_STR,
        )?;
    }

    // Set up the base directory.
    for d in ["dev", "proc", "sys", "tmp", "host"] {
        dir_builder.create(base_dir.join(d))?;
    }

    // Bind-mount lower directories so we can refer to them in overlayfs options
    // in very short file paths because the maximum length of option strings for
    // mount(2) is constrained.
    let mut short_lower_dirs: Vec<String> = Vec::new();

    dir_builder.create(lowers_dir.join("base"))?;
    mount(
        Some(&base_dir),
        &lowers_dir.join("base"),
        NONE_STR,
        BIND_REC,
        NONE_STR,
    )
    .with_context(|| format!("Bind-mounting {} failed", base_dir.display()))?;
    short_lower_dirs.push("base".to_owned());

    for (i, lower_dir) in cfg.lower_dirs.iter().enumerate() {
        let name = i.to_string();
        let path = lowers_dir.join(&name);
        dir_builder.create(&path)?;
        mount(Some(lower_dir), &path, NONE_STR, BIND_REC, NONE_STR)
            .with_context(|| format!("Bind-mounting {} failed", lower_dir.display()))?;
        short_lower_dirs.push(name);
    }

    // Change the current directory temporarily to use relative paths to refer
    // to lower directories.
    let orig_wd = std::env::current_dir()?;
    std::env::set_current_dir(&lowers_dir)?;

    // overlayfs fails to mount if there are 500+ lower layers. Check the
    // condition in advance for better diagnostics.
    ensure!(
        cfg.lower_dirs.len() <= 500,
        "Too many overlayfs lower dirs ({} > 500)",
        cfg.lower_dirs.len()
    );

    // Overlayfs option treats the first lower directory as the least lower
    // directory, while we order filesystem layers in the opposite order.
    let short_lower_dirs_str = short_lower_dirs.into_iter().rev().join(":");

    // Mount overlayfs.
    let overlay_options = format!(
        "upperdir={},workdir={},lowerdir={}",
        upper_dir.display(),
        work_dir.display(),
        short_lower_dirs_str
    );
    mount(
        Some("none"),
        &root_dir,
        Some("overlay"),
        MsFlags::empty(),
        Some::<&str>(&overlay_options),
    )
    .with_context(|| "mounting overlayfs")?;

    // Mount misc file systems.
    mount(
        Some("/dev"),
        &root_dir.join("dev"),
        NONE_STR,
        BIND_REC,
        NONE_STR,
    )
    .with_context(|| "Bind-mounting /dev")?;
    mount(
        Some("/proc"),
        &root_dir.join("proc"),
        Some("proc"),
        MsFlags::empty(),
        NONE_STR,
    )
    .with_context(|| "Bind-mounting /proc")?;
    mount(
        Some("/sys"),
        &root_dir.join("sys"),
        NONE_STR,
        BIND_REC,
        NONE_STR,
    )
    .with_context(|| "Bind-mounting /sys")?;

    for spec in cfg.bind_mounts {
        let target = root_dir.join(spec.mount_path.strip_prefix("/")?);
        // Paths are sometimes provided as relative paths, but we changed directory earlier.
        // Thus, we need to join to the old working directory.
        let source = orig_wd.join(spec.source);
        dir_builder.create(
            target
                .parent()
                .with_context(|| "Can't bind-mount the root directory")?,
        )?;

        // When bind-mounting, the destination must exist.
        if !target.try_exists()? {
            let info = std::fs::metadata(&source).with_context(|| {
                format!("failed to get the metadata of a bind-mount source {source:?}")
            })?;
            if info.is_dir() {
                dir_builder.create(&target)?;
            } else {
                std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .mode(0o755)
                    .open(&target)?;
            }
        }

        // Unfortunately, the unix.MS_RDONLY flag is ignored for bind-mounts.
        // Thus, we mount a bind-mount, then remount it as readonly.
        mount(Some(&source), &target, NONE_STR, MsFlags::MS_BIND, NONE_STR)
            .with_context(|| format!("Failed bind-mounting {source:?} to {target:?}"))?;
        if !spec.rw {
            mount(
                NONE_STR,
                &target,
                NONE_STR,
                MsFlags::MS_REMOUNT
                    .union(MsFlags::MS_BIND)
                    .union(MsFlags::MS_RDONLY),
                NONE_STR,
            )
            .with_context(|| format!("Failed remounting {target:?} as read-only"))?;
        }
    }

    pivot_root(&root_dir, &root_dir.join("host")).with_context(|| "Failed to pivot root")?;

    if !cfg.keep_host_mount {
        // Do a lazy unmount with DETACH. Since the binary is dynamically linked, we still have some
        // file descriptors such as /host/usr/lib/x86_64-linux-gnu/libc.so.6 open.
        umount2("/host", MntFlags::MNT_DETACH).with_context(|| "unmounting host")?;
    }

    let escaped_command = cfg
        .args
        .iter()
        .map(|s| shell_escape::escape(s.to_string_lossy()))
        .join(" ");
    eprintln!("COMMAND(container): {}", &escaped_command);

    let status = {
        let _span = info_span!("run", command = escaped_command).entered();
        Command::new(&cfg.args[0])
            .args(&cfg.args[1..])
            .env_clear()
            .envs(cfg.envs)
            .current_dir(cfg.chdir)
            .status()?
    };

    Ok(status_to_exit_code(&status))
}
