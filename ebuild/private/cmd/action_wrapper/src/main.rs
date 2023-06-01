// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use chrome_trace::{Event, Phase, Trace};
use clap::Parser;
use cliutil::{handle_top_level_result, PROFILES_DIR_ENV};
use fileutil::SafeTempDir;
use processes::status_to_exit_code;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

const PROGRAM_NAME: &str = "action_wrapper";

#[derive(Parser, Debug)]
#[clap(
    about = "Redirect command output to a file and also print it on error.",
    author, version, about, long_about=None, trailing_var_arg = true)]
struct Cli {
    #[arg(help = "File to save stdout and stderr to", long)]
    log: Option<PathBuf>,

    #[arg(help = "File to save profile JSON file to", long)]
    profile: Option<PathBuf>,

    #[arg(help = "Command to run", required = true)]
    command_line: Vec<String>,
}

fn merge_profiles(input_profiles_dir: &Path, output_profile_file: &Path) -> Result<()> {
    let mut merged_trace = Trace::new();

    // Load all profiles and merge events into one trace.
    for entry in std::fs::read_dir(input_profiles_dir)? {
        let entry = entry?;
        let trace = Trace::load(File::open(entry.path())?)?;
        merged_trace.events.extend(trace.events);
    }

    // Compute timestamp offsets from clock_sync metadata events.
    let mut clock_offset_by_process_id: HashMap<i64, f64> = HashMap::new();
    for event in merged_trace.events.iter() {
        if event.phase != Phase::Metadata {
            continue;
        }
        if event.name != "clock_sync" {
            continue;
        }
        let args_value = match &event.args {
            Some(a) => a,
            None => {
                continue;
            }
        };
        let args_object = match args_value.as_object() {
            Some(o) => o,
            None => {
                continue;
            }
        };
        let system_time_number = match args_object.get("system_time") {
            Some(serde_json::Value::Number(n)) => n,
            _ => {
                continue;
            }
        };
        let system_time = match system_time_number.as_f64() {
            Some(f) => f,
            None => {
                continue;
            }
        };
        let offset = system_time - event.timestamp / 1000.0;
        clock_offset_by_process_id.insert(event.process_id, offset);
    }

    // Update timestamps.
    if let Some(min_clock_offset) = clock_offset_by_process_id
        .values()
        .copied()
        .reduce(f64::min)
    {
        for event in merged_trace.events.iter_mut() {
            let clock_offset = match clock_offset_by_process_id.get(&event.process_id) {
                Some(o) => *o,
                None => {
                    // Leave unadjustable entries as-is.
                    continue;
                }
            };
            event.timestamp += clock_offset - min_clock_offset;
        }
    }

    // Also add process_sort_index metadata to ensure processes are sorted in
    // the execution order.
    let mut clock_offsets: Vec<(i64, f64)> = clock_offset_by_process_id.into_iter().collect();
    clock_offsets.sort_by(|(_, a), (_, b)| a.partial_cmp(b).expect("Clock offset is NaN"));

    for (sort_index, (process_id, _)) in clock_offsets.into_iter().enumerate() {
        merged_trace.events.push(Event {
            name: "process_sort_index".to_owned(),
            category: "".to_owned(),
            phase: Phase::Metadata,
            timestamp: 0.0,
            process_id,
            thread_id: 0,
            args: Some(json!({ "sort_index": sort_index })),
        });
    }

    // Save merged traces.
    merged_trace.save(File::create(output_profile_file)?)?;

    Ok(())
}

fn do_main() -> Result<ExitCode> {
    let args = Cli::parse();

    // Always enable Rust backtraces.
    std::env::set_var("RUST_BACKTRACE", "1");

    // Redirect output to a file if `--log` was specified.
    let mut output = if let Some(log_name) = &args.log {
        Some(File::create(log_name)?)
    } else {
        None
    };

    let mut command = Command::new(&args.command_line[0]);
    command.args(&args.command_line[1..]);

    if let Some(output) = &output {
        command
            .stdout(Stdio::from(output.try_clone()?))
            .stderr(Stdio::from(output.try_clone()?));
    }

    let profiles_dir = SafeTempDir::new()?;
    command.env(PROFILES_DIR_ENV, profiles_dir.path());

    let start_time = Instant::now();
    let status = processes::run(&mut command)?;
    let elapsed = start_time.elapsed();

    let message = if let Some(signal_num) = status.signal() {
        let signal_name = match nix::sys::signal::Signal::try_from(signal_num) {
            Ok(signal) => signal.to_string(),
            Err(_) => signal_num.to_string(),
        };
        format!(
            "{}: Command killed with signal {} in {:.1}s",
            PROGRAM_NAME,
            signal_name,
            elapsed.as_secs_f32()
        )
    } else if let Some(code) = status.code() {
        format!(
            "{}: Command exited with code {} in {:.1}s",
            PROGRAM_NAME,
            code,
            elapsed.as_secs_f32()
        )
    } else {
        unreachable!("Unexpected ExitStatus: {:?}", status);
    };

    if let Some(output) = &mut output {
        writeln!(output, "{}", message)?;
    } else {
        eprintln!("{}", message);
    }

    // If the command failed, then print saved output on the stderr.
    if !status.success() {
        if let Some(log_name) = &args.log {
            let mut read_file = File::open(log_name)?;
            std::io::copy(&mut read_file, &mut std::io::stderr())?;
        }
    }

    if let Some(profile_file) = args.profile {
        merge_profiles(profiles_dir.path(), &profile_file)?;
    }

    // Propagate the exit status of the command.
    Ok(status_to_exit_code(&status))
}

fn main() -> ExitCode {
    // We don't use `cli_main` to avoid emitting the preamble logs because
    // action_wrapper must queue stdout/stderr until it sees the wrapped program
    // to exit abnormally. This means we don't log the arguments passed to
    // action_wrapper itself, but the wrapped program should soon print one with
    // `cli_main`.
    let result = do_main();
    handle_top_level_result(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_process() -> Result<()> {
        processes::run(&mut Command::new("true"))?;
        Ok(())
    }

    #[test]
    fn runs_failed_process() -> Result<()> {
        processes::run(&mut Command::new("false"))?;
        Ok(())
    }
}
