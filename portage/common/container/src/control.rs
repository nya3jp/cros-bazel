// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use nix::{
    errno::Errno,
    sys::select::FdSet,
    sys::signal::{signal, SigHandler, Signal},
};
use scopeguard::defer;
use std::os::unix::io::{AsRawFd, RawFd};
use std::{fs::OpenOptions, io::Read, path::PathBuf};

pub(crate) struct ControlChannel {
    join_handle: Option<std::thread::JoinHandle<()>>,
    tx: RawFd,
}

impl ControlChannel {
    fn reset_controlling_terminal() -> Result<()> {
        let pgid = nix::unistd::getpgid(None)?;

        // SIGTTOU will be generated when a background processes tries to write to
        // the terminal. Since we are writing a new tpgid to the terminal, we need to
        // ignore SIGTTOU, otherwise we get suspended.
        let old_handler = unsafe { signal(Signal::SIGTTOU, SigHandler::SigIgn) }.unwrap();
        defer! {
            unsafe { signal(Signal::SIGTTOU, old_handler) }.unwrap();
        }
        nix::unistd::tcsetpgrp(0, pgid)?;
        Ok(())
    }

    fn read_fifo(path: PathBuf, rx: RawFd) -> Result<()> {
        // We open RDWR so that we always keep a write handle to the FIFO. This
        // makes the open call not block waiting for a writer to open the FIFO. It
        // also allows writers to open/close the FIFO without causing the reader (us)
        // to close.
        let mut fifo = OpenOptions::new().read(true).write(true).open(path)?;
        let fifo_fd = fifo.as_raw_fd();
        loop {
            let mut read_fds = FdSet::new();
            read_fds.insert(fifo_fd);
            read_fds.insert(rx);

            match nix::sys::select::select(None, &mut read_fds, None, None, None) {
                Ok(_) => {}
                Err(Errno::EINTR) => continue,
                Err(e) => return Err(e.into()),
            }

            if read_fds.contains(fifo_fd) {
                let mut buf: [u8; 1] = [0];
                fifo.read_exact(&mut buf)?;
                match buf[0] as char {
                    't' => ControlChannel::reset_controlling_terminal()
                        .with_context(|| "Failed to update terminal pgid")?,
                    c => bail!("Unknown control command: {c}"),
                }
            }

            if read_fds.contains(rx) {
                nix::unistd::close(rx)?;
                return Ok(());
            }
        }
    }

    pub fn new(path: PathBuf) -> Result<Self> {
        nix::unistd::mkfifo(&path, nix::sys::stat::Mode::from_bits(0o666).unwrap())?;
        let (tx, rx) = nix::unistd::pipe()?;
        Ok(Self {
            join_handle: Some(std::thread::spawn(move || {
                if let Err(e) = Self::read_fifo(path, rx) {
                    eprintln!("Failed to read fifo: {e}");
                }
            })),
            tx,
        })
    }
}

impl Drop for ControlChannel {
    fn drop(&mut self) {
        nix::unistd::close(self.tx).unwrap();
        self.join_handle.take().unwrap().join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use fileutil::SafeTempDir;

    use super::*;

    use std::time::Duration;

    #[test]
    pub fn creates_control_channel() -> Result<()> {
        let tmp_dir = SafeTempDir::new()?;
        let path = tmp_dir.path().join("control");
        let _control = ControlChannel::new(path.clone())?;
        assert!(path.try_exists()?);
        std::thread::sleep(Duration::from_millis(50));
        std::fs::write(&path, "t")?;
        std::thread::sleep(Duration::from_millis(50));

        Ok(())
    }
}
