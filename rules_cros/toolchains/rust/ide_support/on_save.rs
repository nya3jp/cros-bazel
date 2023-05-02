// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{Context, Result};
use by_address::ByAddress;
use log::{debug, info};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

static POLL_INTERVAL: Duration = Duration::from_secs(2);

struct RustcOutputFileContent {
    modified: SystemTime,
    bytes: Vec<u8>,
}

struct RustcOutputFile {
    path: PathBuf,
    is_test: bool,
    // The content is cached, and updated whenever the mtime of the file is updated.
    content: RefCell<Option<RustcOutputFileContent>>,
}

impl RustcOutputFile {
    pub fn new(path: &Path) -> RustcOutputFile {
        Self {
            is_test: path.to_string_lossy().to_string().contains("/test-"),
            path: path.into(),
            content: RefCell::new(None),
        }
    }

    /// Updates the file content. Returns true if the contents have changed.
    pub fn refresh(&self) -> Result<bool> {
        debug!("Checking contents of {:?}", self.path);
        match (|| -> std::io::Result<bool> {
            let modified = std::fs::metadata(&self.path)?.modified()?;
            let changed = Some(modified) != self.modified();
            if changed {
                info!("Contents of {:?} have changed", self.path);
                self.content.replace(Some(RustcOutputFileContent {
                    modified,
                    bytes: std::fs::read(&self.path)?,
                }));
            }
            Ok(changed)
        })() {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(self.content.replace(None).is_some())
            }
            other => other,
        }
        .with_context(|| format!("While refreshing {:?}", self.path))
    }

    pub fn modified(&self) -> Option<SystemTime> {
        self.content.borrow().as_ref().map(|c| c.modified)
    }
}

struct SourceFile<'a> {
    // This is useful for debugging.
    #[allow(dead_code)]
    path: PathBuf,
    files: Vec<&'a RustcOutputFile>,
}

impl<'a> SourceFile<'_> {
    /// Outputs the rustc output file for the specified source file.
    /// We may have multiple output files generated (eg. both hello_world and
    /// hello_world_test depend on hello_world.rs). In this case, if we output
    /// all of them, we can get all kinds of weird outputs. For example:
    /// * You compile the non-test successfully. However, the IDE still shows
    ///   an error because you also read the failing results from
    ///   test.rustc-output.
    /// * You compile both successfully. The IDE shows two type annotations
    ///   each object.
    /// To solve this, we take the most recently compiled one and output only
    /// that.
    /// In the case of a tie, we take the test, as it may have annotations for
    /// parts of the file that the non-test doesn't.
    pub fn rustc_output(&self) -> Option<&'_ RustcOutputFile> {
        self.files
            .iter()
            .filter_map(|f| Some((*f, (f.modified()?, f.is_test))))
            .max_by(|(_, lhs), (_, rhs)| lhs.cmp(rhs))
            .map(|(f, _)| f)
    }
}

pub fn start_watcher(src_to_rustc_output: &HashMap<PathBuf, HashSet<PathBuf>>) -> Result<()> {
    let rustc_output_files: HashMap<&PathBuf, RustcOutputFile> = src_to_rustc_output
        .values()
        .flatten()
        .collect::<HashSet<&PathBuf>>()
        .iter()
        .map(|path| (*path, RustcOutputFile::new(path)))
        .collect();

    let srcs: Vec<SourceFile> = src_to_rustc_output
        .iter()
        .map(|(src, rustc_output_paths)| {
            Ok(SourceFile {
                path: src.clone(),
                files: rustc_output_paths
                    .iter()
                    .map(|path| Ok(rustc_output_files.get(path).context("File missing")?))
                    .collect::<Result<Vec<_>>>()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    info!(
        "Waiting for file content updates on {} targets",
        rustc_output_files.len()
    );

    loop {
        let target_ts = Instant::now() + POLL_INTERVAL;
        let any_changed = rustc_output_files
            .values()
            .map(|f| f.refresh())
            .collect::<Result<Vec<bool>>>()?
            .iter()
            .any(|x| *x);

        if any_changed {
            info!("File contents have changed. Updating rust-analyzer.");
            info!("----------------------------------------------------------");
            // Tell rust-analyzer that the things we output in the last
            // iteration of the loop is no longer valid.
            let printed_files = srcs
                .iter()
                .map(SourceFile::rustc_output)
                .flatten()
                // Use ByAddress to hash by pointer to avoid comparing the file contents.
                .map(ByAddress)
                .collect::<BTreeSet<ByAddress<&RustcOutputFile>>>();

            let mut out_f = std::fs::File::create("rust-project-flycheck.txt")?;
            for f in printed_files {
                info!("Printing {:?}", f.path);
                if let Some(content) = f.content.borrow().as_ref() {
                    out_f.write_all(&content.bytes)?;
                }
            }
            // This program needs to invoke flycheck, but that can only be
            // done from the vscode extension. Thus, the synchronisation
            // method is:
            // * This program writes a line to stdout, and waits for stdin.
            // * Extension reads line, and decides to update the flychecks
            // * When flychecks are done running, extension writes to stdin.
            // * This extension picks up the change.
            println!("Update flycheck");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
        }
        std::thread::sleep(target_ts - Instant::now());
    }
}
