// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use std::io::Write;

pub fn do_main() -> Result<()> {
    println!("stdout");
    // Ensure that stdout comes before stderr, so that the merged stream is deterministic.
    std::io::stdout().flush()?;

    eprintln!("stderr");
    log::debug!("log at level debug");
    log::info!("log at level info");

    if let Ok(value) = std::env::var("ERROR") {
        bail!("{}", value);
    }

    if let Ok(value) = std::env::var("THREAD_PANIC") {
        std::thread::spawn(move || panic!("{}", value)).join().ok();
    }

    if let Ok(value) = std::env::var("MAIN_PANIC") {
        panic!("{}", value);
    }

    if let Ok(value) = std::env::var("PROCESS_EXIT") {
        std::process::exit(value.parse::<i32>()?)
    }

    Ok(())
}

pub fn main() -> std::process::ExitCode {
    cliutil::cli_main(
        do_main,
        cliutil::ConfigBuilder::new()
            .log_command_line(false)
            .build()
            .unwrap(),
    )
}
