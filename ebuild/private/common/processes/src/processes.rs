// Copyright 2023 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use nix::sys::signal::Signal;
use signal_hook::{
    consts::signal::{SIGCHLD, SIGINT, SIGTERM},
    iterator::Signals,
};
use std::process::{Command, Output};

// run runs a child process, with some special signal handling:
//   - Forwards SIGTERM to the child processes
//   - Ignores SIGINT while the processes is running. SIGINT is normally generated
//     by the terminal when Ctrl+C is pressed. The signal is sent to all processes
//     in the foreground processes group. This means that the child processes
//     should receive the signal by default so we don't need to forward it. One
//     exception is if the child puts itself into a different processes group, but
//     we want to avoid that.
pub fn run(cmd: &mut Command) -> Result<Output> {
    // Register the signal handler before spawning the process to ensure we don't drop any signals.
    let mut signals = Signals::new(&[SIGCHLD, SIGINT, SIGTERM])?;

    let mut child = cmd.spawn()?;

    for signal in signals.forever() {
        match signal as libc::c_int {
            SIGCHLD => match &child.try_wait()? {
                Some(_) => return Ok(child.wait_with_output()?),
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

pub fn run_and_check(cmd: &mut Command) -> Result<Output> {
    let out = run(cmd)?;
    if !out.status.success() {
        bail!(
            "Command {cmd:?} failed to run. Got stderr {}",
            std::str::from_utf8(&out.stderr)?
        );
    }
    Ok(out)
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
