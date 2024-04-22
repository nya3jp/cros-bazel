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
use archive_logs::archive_logs;
use build_event_processor::BuildEventProcessor;
use clap::Parser;
use prebuilts::compute_prebuilts;
use proto::build_event_stream::BuildEvent;

mod archive_logs;
mod build_event_processor;
mod prebuilts;
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
    #[arg(long, required = true)]
    build_events_jsonl: PathBuf,

    /// Path to the Bazel workspace where bazel-* symlinks are located.
    /// [default: $BUILD_WORKSPACE_DIRECTORY]
    #[arg(long, default_value = get_default_workspace_dir(), hide_default_value = true)]
    workspace: PathBuf,

    /// If set, creates a tarball containing all logs created in the build to this file path.
    /// Compression algorithm is selected by the file name extension (using GNU tar's
    /// --auto-compress option).
    #[arg(long)]
    archive_logs: Option<PathBuf>,

    /// If set, a .bzl file will be generated that contains --@portage//<package>_prebuilt
    /// flags pointing to the CAS for the packages specified in the BEP file..
    #[arg(long)]
    prebuilts: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let events = load_build_events_jsonl(&args.build_events_jsonl)?;
    let processor = BuildEventProcessor::from(&events);

    if let Some(output_path) = &args.archive_logs {
        archive_logs(output_path, &args.workspace, &processor)?;
    }

    if let Some(output_path) = &args.prebuilts {
        compute_prebuilts(output_path, &args.workspace, &processor)?;
    }

    Ok(())
}
