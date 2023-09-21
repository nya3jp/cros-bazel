// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::Result;
use std::iter::Iterator;
use std::process::Command;

static TIMESTAMP_REGEX: &str = r"^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d.\d{6}Z\s*";
static INFO: &str = "INFO demo: log at level info";

fn normalize_file(s: &[u8]) -> Result<Vec<String>> {
    let s = std::str::from_utf8(s)?;
    // Remove all ansi color control signals.
    let s = regex::Regex::new("\u{1b}\\[\\d+m")
        .unwrap()
        .replace_all(s, "");

    // Remove the timestamps.
    // let re = regex::RegexBuilder::new(TIMESTAMP_REGEX).multi_line(true).build().unwrap();
    let re = regex::Regex::new(TIMESTAMP_REGEX).unwrap();
    Ok(s.lines()
        .map(|line| re.replace_all(line, "").trim().to_string())
        .collect())
}

fn to_string_vec(slice: &[&str]) -> Vec<String> {
    slice.iter().map(|s| s.to_string()).collect()
}

#[derive(PartialEq, Debug)]
struct Output {
    success: bool,
    stdout: Vec<String>,
    stderr: Vec<String>,
}

fn run_cli_main_test(env: &[(&str, &str)]) -> Result<Output> {
    let r = runfiles::Runfiles::create()?;
    let demo = r.rlocation("cros/bazel/portage/common/cliutil/testdata/demo");

    let mut cmd = Command::new(demo);
    cmd.env("RUST_BACKTRACE", "0");
    for (k, v) in env {
        cmd.env(k, v);
    }
    let output = cmd.output()?;
    Ok(Output {
        success: output.status.success(),
        stdout: normalize_file(&output.stdout)?,
        stderr: normalize_file(&output.stderr)?,
    })
}

#[test]
fn test_simple() -> Result<()> {
    assert_eq!(
        run_cli_main_test(&[])?,
        Output {
            success: true,
            stdout: to_string_vec(&["stdout"]),
            stderr: to_string_vec(&["stderr", INFO]),
        }
    );
    Ok(())
}

#[test]
fn test_redirection_on_success() -> Result<()> {
    let out = fileutil::SafeTempDir::new()?;
    let redirected = out.path().join("redirected");
    assert_eq!(
        run_cli_main_test(&[
            ("CROS_BAZEL_STDIO_REDIRECT", redirected.to_str().unwrap()),
        ])?,
        Output {
            success: true,
            stdout: [].into(),
            stderr: [].into(),
        }
    );

    Ok(())
}

#[test]
fn test_redirection_on_failure() -> Result<()> {
    let out = fileutil::SafeTempDir::new()?;
    let redirected = out.path().join("redirected");
    assert_eq!(
        run_cli_main_test(&[
            ("ERROR", "unknown error"),
            ("CROS_BAZEL_STDIO_REDIRECT", redirected.to_str().unwrap()),
        ])?,
        Output {
            success: true,
            stdout: [].into(),
            stderr: to_string_vec(&["stdout", "stderr", INFO, "FATAL: demo: unknown error"]),
        }
    );

    Ok(())
}
