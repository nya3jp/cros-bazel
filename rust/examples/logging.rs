// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};

fn do_main() -> Result<()> {
    log::error!("logging at level error");
    log::warn!("logging at level warn");
    log::info!("logging at level info");
    log::debug!("logging at level debug");
    log::trace!("logging at level trace");

    if let Ok(msg) = std::env::var("LOG_FAIL_MSG") {
        bail!("Failed: {msg}")
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    cliutil::cli_main(
        do_main,
        cliutil::ConfigBuilder::new()
            // Not required, just to show how you can change the logging config.
            .logging(cliutil::LoggingConfig::from_env().unwrap())
            .log_command_line(false)
            .build()
            .unwrap(),
    )
}
