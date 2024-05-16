// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};

use std::process::Command;

/// A helper trait to implement `Command::run_ok`.
pub trait CommandRunOk {
    /// Runs a command and ensures it exits with success.
    fn run_ok(&mut self) -> Result<()>;
}

impl CommandRunOk for Command {
    fn run_ok(&mut self) -> Result<()> {
        let status = self.status()?;
        if !status.success() {
            bail!("Command exited with {:?}", status);
        }
        Ok(())
    }
}
