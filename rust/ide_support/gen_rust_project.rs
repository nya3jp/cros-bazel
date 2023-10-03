// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use clap::Parser;
use log::{debug, info};
use on_save::start_watcher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
struct Args {
    #[arg(long, num_args=1..)]
    files: Vec<PathBuf>,
}

fn run_command(mut cmd: process::Command) -> Result<process::Output> {
    info!("Running {cmd:?}");
    let out = cmd.stderr(process::Stdio::inherit()).output()?;
    if !out.status.success() {
        bail!("{cmd:?} exited with exit status {}", out.status,);
    }
    Ok(out)
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;
    env_logger::init();
    let r = runfiles::Runfiles::create()?;

    let gen_rust_project = r.rlocation("rules_rust/tools/rust_analyzer/gen_rust_project");
    let cquery_path = r
        .rlocation("cros/bazel/rust/ide_support/get_outputs.bzl")
        .to_string_lossy()
        .to_string();

    let workspace_dir = PathBuf::from(std::env::var("BUILD_WORKSPACE_DIRECTORY")?);
    std::env::set_current_dir(&workspace_dir)?;

    eprintln!("Resolving packages");
    let pkgs = args
        .files
        .iter()
        .filter_map(|p| {
            info!("Determining files relevant to {p:?}");
            // If you have a standard library file open, it may not be under
            // the workspace.
            let p = match p.strip_prefix(&workspace_dir) {
                Err(_) => return None,
                Ok(p) => p,
            };

            Some((|| {
                let mut pkg: &Path = p;
                loop {
                    debug!("Checking directory {pkg:?}");
                    pkg = pkg.parent().with_context(|| {
                        format!("{p:?} must be in a tree containing a BUILD.bazel file")
                    })?;
                    if pkg.join("BUILD.bazel").try_exists()? || pkg.join("BUILD").try_exists()? {
                        break;
                    }
                }
                let pkg_label = if let Some(std::path::Component::RootDir) = pkg.components().next()
                {
                    "//:all".to_string()
                } else {
                    format!("//{}:all", pkg.to_string_lossy())
                };
                info!("Resolved to package {pkg_label}");
                Ok(pkg_label)
            })())
        })
        .collect::<Result<HashSet<String>>>()?;

    // gen_rust_project will look like:
    // <OUTPUT_BASE>/execroot/_main/bazel-out/k8-dbg/bin/.../gen_rust_project.runfiles/...
    let mut execroot: &Path = &gen_rust_project;
    while execroot.file_name() != Some(std::ffi::OsStr::new("execroot")) {
        execroot = execroot.parent().with_context(|| {
            format!("There must be a directory below {gen_rust_project:?} named execroot")
        })?;
    }

    eprintln!("Generating rust-project.json");
    let mut cmd = process::Command::new(&gen_rust_project);
    cmd.arg("--workspace");
    cmd.arg(
        std::env::var_os("BUILD_WORKSPACE_DIRECTORY")
            .context("gen_rust_project must be run via 'bazel run'")?,
    );
    cmd.arg("--execution-root");
    cmd.arg(&execroot);
    cmd.arg("--output-base");
    cmd.arg(
        &execroot
            .parent()
            .context("The execroot must have a parent directory")?,
    );
    cmd.args(&pkgs);
    run_command(cmd)?;

    eprintln!("Generated rust-project.json");

    eprintln!("Determining which files we should watch");
    let mut src_to_rustc_outputs: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    for pkg in &pkgs {
        let mut cmd = process::Command::new("bazel");
        cmd.args([
            "cquery",
            "--output",
            "starlark",
            "--starlark:file",
            &cquery_path,
            &pkg,
        ])
        .current_dir(&workspace_dir)
        .stdout(process::Stdio::piped());

        let stdout = String::from_utf8(run_command(cmd)?.stdout)?;
        let lines = stdout.lines().filter(|line| !line.is_empty());

        for line in lines {
            for (src, output) in serde_json::from_str::<HashMap<PathBuf, PathBuf>>(line)? {
                src_to_rustc_outputs.entry(src).or_default().insert(output);
            }
        }
    }
    eprintln!("Watching files");
    start_watcher(&src_to_rustc_outputs)?;
    Ok(())
}
