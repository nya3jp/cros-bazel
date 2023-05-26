// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use nix::sys::signal::Signal;
use signal_hook::{
    consts::signal::{SIGCHLD, SIGINT, SIGTERM},
    iterator::Signals,
};
use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, ExitCode, ExitStatus},
};
use tracing::instrument;

// run runs a child process, with some special signal handling:
//   - Forwards SIGTERM to the child processes
//   - Ignores SIGINT while the processes is running. SIGINT is normally generated
//     by the terminal when Ctrl+C is pressed. The signal is sent to all processes
//     in the foreground processes group. This means that the child processes
//     should receive the signal by default so we don't need to forward it. One
//     exception is if the child puts itself into a different processes group, but
//     we want to avoid that.
#[instrument(skip_all, fields(command = %cmd.get_program().to_string_lossy()))]
pub fn run(cmd: &mut Command) -> Result<ExitStatus> {
    // Register the signal handler before spawning the process to ensure we don't drop any signals.
    let mut signals = Signals::new([SIGCHLD, SIGINT, SIGTERM])?;

    let mut child = cmd.spawn()?;

    for signal in signals.forever() {
        match signal {
            SIGCHLD => match &child.try_wait()? {
                Some(status) => return Ok(*status),
                None => continue,
            },
            SIGINT => {}
            SIGTERM => nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(child.id().try_into()?),
                Signal::SIGTERM,
            )?,
            _ => unreachable!(),
        }
    }
    unreachable!()
}

#[instrument(skip_all, fields(command = %cmd.get_program().to_string_lossy()))]
pub fn run_and_check(cmd: &mut Command) -> Result<()> {
    let stauts = run(cmd)?;
    if !stauts.success() {
        bail!("Command {cmd:?} failed with {stauts}");
    }

    Ok(())
}

/// Converts [`ExitStatus`] to [`ExitCode`] following the POSIX shell
/// convention.
///
/// It panics [`ExitStatus`] does not represent a status of an exiting process
/// (e.g. process being stopped or continued). This won't happen as long as you
/// get [`ExitStatus`] from [`std::process`] methods, such as
/// [`Command::status`], [`Command::output`],
/// [`Child::wait`](std::process::Child::wait).
pub fn status_to_exit_code(status: &ExitStatus) -> ExitCode {
    if let Some(code) = status.code() {
        ExitCode::from(code as u8)
    } else if let Some(signal) = status.signal() {
        ExitCode::from(128 + signal as u8)
    } else {
        panic!("ExitStatus does not represent process exit: {:?}", status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_process() -> Result<()> {
        run_and_check(&mut Command::new("true"))?;
        Ok(())
    }

    #[test]
    fn runs_failed_process() -> Result<()> {
        run(&mut Command::new("false"))?;
        assert!(run_and_check(&mut Command::new("false")).is_err());
        Ok(())
    }
}
