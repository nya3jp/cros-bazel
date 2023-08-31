// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use std::{fs::File, os::fd::AsRawFd, os::fd::FromRawFd, os::fd::OwnedFd, path::Path};

/// Redirects stdout and stderr to the specified file, and returns the saved
/// stdout/stderr file descriptors.
fn redirect_stdout_stderr(output: &File) -> Result<(OwnedFd, OwnedFd)> {
    let stdout_fd = std::io::stdout().as_raw_fd();
    let saved_stdout_fd = nix::fcntl::fcntl(stdout_fd, nix::fcntl::F_DUPFD_CLOEXEC(3))?;
    let saved_stdout = unsafe { OwnedFd::from_raw_fd(saved_stdout_fd) };

    let stderr_fd = std::io::stderr().as_raw_fd();
    let saved_stderr_fd = nix::fcntl::fcntl(stderr_fd, nix::fcntl::F_DUPFD_CLOEXEC(3))?;
    let saved_stderr = unsafe { OwnedFd::from_raw_fd(saved_stderr_fd) };

    let output_fd = output.as_raw_fd();
    nix::unistd::dup2(output_fd, stdout_fd)?;
    nix::unistd::dup2(output_fd, stderr_fd)?;

    Ok((saved_stdout, saved_stderr))
}

pub struct StdioRedirector {
    file: File,
    saved_stdout: OwnedFd,
    saved_stderr: File,
}

impl StdioRedirector {
    /// Redirects stdout and stderr to the specified path.
    pub fn new(path: &Path) -> Result<Self> {
        let file = File::create(path).context("Failed to create the log file")?;
        let (saved_stdout, saved_stderr) =
            redirect_stdout_stderr(&file).context("Failed to redirect stdout/stderr")?;

        Ok(Self {
            file,
            saved_stdout,
            saved_stderr: saved_stderr.into(),
        })
    }

    /// Prints the contents of the file to the real stderr.
    /// Also consumes the redirector, which restores the original stdout/stderr.
    pub fn flush_to_real_stderr(mut self) -> Result<()> {
        // Reopen the file to get an independent seek position.
        let read_file = File::open(format!("/proc/self/fd/{}", self.file.as_raw_fd()));
        std::io::copy(&mut read_file?, &mut self.saved_stderr)?;
        Ok(())
    }
}

impl Drop for StdioRedirector {
    fn drop(&mut self) {
        nix::unistd::dup2(self.saved_stdout.as_raw_fd(), std::io::stdout().as_raw_fd()).unwrap();
        nix::unistd::dup2(self.saved_stderr.as_raw_fd(), std::io::stderr().as_raw_fd()).unwrap();
    }
}
