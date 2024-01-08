// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use clap::Parser;
use cliutil::{cli_main, handle_top_level_result, log_current_command_line};
use fileutil::SafeTempDir;
use itertools::Itertools;
use nix::{
    errno::Errno,
    mount::MntFlags,
    mount::{mount, umount2, MsFlags},
    sched::{unshare, CloneFlags},
    sys::socket::{socket, AddressFamily, SockFlag, SockProtocol, SockType},
    unistd::pivot_root,
};
use processes::status_to_exit_code;
use run_in_container_lib::RunInContainerConfig;
use std::{
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};
use tracing::info_span;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

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
        let _guard = cliutil::LoggingConfig {
            trace_file: None,
            log_file: None,
            console_logger: Some(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            ),
        }
        .setup()
        .unwrap();
        log_current_command_line();
        let result = || -> Result<_> {
            enter_namespace(RunInContainerConfig::deserialize_from(&args.config)?)
        }();
        handle_top_level_result(result)
    } else {
        cli_main(
            || continue_namespace(RunInContainerConfig::deserialize_from(&args.config)?),
            Default::default(),
        )
    }
}

fn enter_namespace(cfg: RunInContainerConfig) -> Result<ExitCode> {
    let r = runfiles::Runfiles::create()?;
    let dumb_init_path = r.rlocation("files/dumb_init");

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

    // Remount all file systems as private so that we never interact with the
    // original namespace. This is needed when the current process is privileged
    // and did not enter an unprivileged user namespace.
    mount(
        Some(""),
        "/",
        Some(""),
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        Some(""),
    )
    .context("Failed to remount file systems as private")?;

    if !cfg.allow_network_access {
        enable_loopback_networking()?;
    }

    // Mount /proc. It is done here, not in the container crate, because we need
    // to enter a PID namespace to mount one.
    mount(
        Some("/proc"),
        &cfg.root_dir.join("proc"),
        Some("proc"),
        MsFlags::empty(),
        Some(""),
    )
    .context("Failed to mount /proc")?;

    // We switch into the root dir so that pivot_root will automatically update
    // our CWD to point to the new root.
    std::env::set_current_dir(&cfg.root_dir)
        .with_context(|| format!("Failed to `cd {}`", cfg.root_dir.display()))?;

    pivot_root(".", &cfg.root_dir.join("host")).context("Failed to pivot root")?;

    if !cfg.keep_host_mount {
        // Do a lazy unmount with DETACH. Since the binary is dynamically linked, we still have some
        // file descriptors such as /host/usr/lib/x86_64-linux-gnu/libc.so.6 open.
        umount2("/host", MntFlags::MNT_DETACH).context("Failed to unmount /host")?;
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
