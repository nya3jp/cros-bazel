// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Context, Result};
use clap::Parser;
use commands::{
    archive_logs::archive_logs, diagnose_cache_hits::diagnose_cache_hits,
    prebuilts::compute_prebuilts,
};
use processors::{build_event::BuildEventProcessor, execlog::ExecLogProcessor};
use prost::Message;
use proto::{build_event_stream::BuildEvent, spawn::ExecLogEntry};

mod commands;
mod processors;
mod proto;

/// Loads a newline-deliminated JSON file containing Build Event Protocol data.
fn load_build_events_jsonl(path: &Path) -> Result<Vec<BuildEvent>> {
    let f = File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let f = BufReader::new(f);

    let mut events: Vec<BuildEvent> = Vec::new();
    for (i, line) in f.lines().enumerate() {
        let line = line.with_context(|| format!("Failed to parse {}", path.display()))?;
        let event = serde_json::from_str(&line)
            .with_context(|| format!("Failed to parse {}: line {}", path.display(), i + 1))?;
        events.push(event);
    }

    Ok(events)
}

fn load_compact_execlog(path: &Path) -> Result<Vec<ExecLogEntry>> {
    let data = std::fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let data = zstd::decode_all(data.as_slice())
        .with_context(|| format!("Failed to decode {}", path.display()))?;

    let mut buf = data.as_slice();
    let mut entries: Vec<ExecLogEntry> = Vec::new();
    while !buf.is_empty() {
        let size = prost::decode_length_delimiter(&mut buf)
            .context("Corrupted execlog: failed to decode message length")?;
        let entry = ExecLogEntry::decode(&buf[..size])
            .context("Corrupted execlog: failed to deserialize ExecLogEntry")?;
        entries.push(entry);
        buf = &buf[size..];
    }

    Ok(entries)
}

fn get_default_workspace_dir() -> &'static OsStr {
    static CACHE: OnceLock<OsString> = OnceLock::new();
    CACHE.get_or_init(|| std::env::var_os("BUILD_WORKSPACE_DIRECTORY").unwrap_or(".".into()))
}

/// Bazel build result postprocessor.
///
/// This program is responsible for translating build artifacts left in bazel-out/ to files that
/// can be interpreted by other programs outside of //bazel. Since bazel-out/ contains a lot of
/// implementation details internal to //bazel, external programs should not try to interpret them;
/// otherwise they can break for subtle changes to the internal layout. This program works as
/// the bridge for the API boundary.
#[derive(Parser, Debug)]
struct Args {
    /// Path to the Build Event Protocol JSONL file.
    #[arg(long)]
    build_events_jsonl: Option<PathBuf>,

    /// Path to the compact execlog file.
    #[arg(long)]
    compact_execlog: Option<PathBuf>,

    /// Path to the Bazel workspace where bazel-* symlinks are located.
    /// [default: $BUILD_WORKSPACE_DIRECTORY]
    #[arg(long, default_value = get_default_workspace_dir(), hide_default_value = true)]
    workspace: PathBuf,

    /// If set, creates a tarball containing all logs created in the build to this file path.
    /// Compression algorithm is selected by the file name extension (using GNU tar's
    /// --auto-compress option).
    #[arg(long, requires = "build_events_jsonl")]
    archive_logs: Option<PathBuf>,

    /// If set, a .bzl file will be generated that contains --@portage//<package>_prebuilt
    /// flags pointing to the CAS for the packages specified in the BEP file..
    #[arg(long, requires = "build_events_jsonl")]
    prebuilts: Option<PathBuf>,

    /// If set, diagnoses cache hits from execlog and write human-readable results to the specified
    /// file.
    #[arg(long, requires = "compact_execlog")]
    diagnose_cache_hits: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let events = if let Some(path) = &args.build_events_jsonl {
        Some(load_build_events_jsonl(path)?)
    } else {
        None
    };
    let events_processor = events.as_ref().map(BuildEventProcessor::from);

    let execlog = if let Some(path) = &args.compact_execlog {
        Some(load_compact_execlog(path)?)
    } else {
        None
    };
    let execlog_processor = execlog.as_ref().map(ExecLogProcessor::from);

    if let Some(output_path) = &args.archive_logs {
        archive_logs(
            output_path,
            &args.workspace,
            events_processor
                .as_ref()
                .context("--build-events-jsonl must be set")?,
        )?;
    }

    if let Some(output_path) = &args.prebuilts {
        compute_prebuilts(
            output_path,
            &args.workspace,
            events_processor
                .as_ref()
                .context("--build-events-jsonl must be set")?,
        )?;
    }

    if let Some(output_path) = &args.diagnose_cache_hits {
        diagnose_cache_hits(
            output_path,
            execlog_processor
                .as_ref()
                .context("--compact-execlog must be set")?,
        )?;
    }

    Ok(())
}
