// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use std::{
    collections::HashSet,
    ffi::{OsStr, OsString},
    fs,
    os::unix::ffi::OsStrExt,
};

fn os_str_strip_prefix<'a>(s: &'a OsStr, prefix: &OsStr) -> Option<&'a OsStr> {
    if s.as_bytes().starts_with(prefix.as_bytes()) {
        Some(OsStr::from_bytes(&s.as_bytes()[prefix.as_bytes().len()..]))
    } else {
        None
    }
}

fn read_param_file(
    path: &OsStr,
    file_prefix_symbol: &str,
    out: &mut Vec<OsString>,
    visited_files: &mut HashSet<OsString>,
) -> Result<()> {
    if !visited_files.insert(path.into()) {
        bail!("parameter file loop detected");
    }
    let contents = fs::read_to_string(path)?;
    for (lineno, line) in contents.lines().enumerate() {
        expand_single_param(line.into(), file_prefix_symbol, out, visited_files)
            .with_context(|| format!("line {lineno}"))?;
    }
    Ok(())
}

fn expand_single_param(
    param: OsString,
    file_prefix_symbol: &str,
    out: &mut Vec<OsString>,
    visited_files: &mut HashSet<OsString>,
) -> Result<()> {
    match os_str_strip_prefix(&param, OsStr::from_bytes(file_prefix_symbol.as_bytes())) {
        Some(path) => read_param_file(path, file_prefix_symbol, out, visited_files)
            .with_context(|| format!("error processing parameter {:?}", param)),
        None => {
            out.push(param);
            Ok(())
        }
    }
}

/// Expand command line arguments containing parameter files, denoted by
/// an argument starting with the character `@`.
///
/// For an argument `@file`, the arguments read from the file are inserted in
/// place of the original @file argument.
/// Arguments in the parameter file are terminated by newlines.
///
/// This is compatible with Bazel's
/// `set_param_file_format("multiline")` and `use_param_file("@%s")`:
/// https://bazel.build/rules/lib/builtins/Args#set_param_file_format
/// https://bazel.build/rules/lib/builtins/Args#use_param_file
fn expand_params<I>(itr: I) -> Result<Vec<OsString>>
where
    I: IntoIterator,
    I::Item: Into<OsString> + Clone,
{
    let mut expanded = Vec::new();
    let mut visited_files = HashSet::new();

    for item in itr {
        let param: OsString = item.into();
        expand_single_param(param, "@", &mut expanded, &mut visited_files)?;
    }

    Ok(expanded)
}

pub fn expanded_args_os() -> Result<Vec<OsString>> {
    expand_params(std::env::args_os())
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};

    use super::expand_params;

    #[test]
    fn test_expand() {
        let tempdir = tempfile::tempdir().unwrap();
        let param_file = tempdir.path().join("param_file");
        std::fs::write(&param_file, "bar\nbaz\n\n").unwrap();

        assert_eq!(
            expand_params([
                OsString::from("foo"),
                [OsStr::new("@"), param_file.as_os_str()].join(OsStr::new("")),
                OsString::from("--flag=@value"),
            ])
            .unwrap(),
            [
                OsStr::new("foo"),
                OsStr::new("bar"),
                OsStr::new("baz"),
                OsStr::new(""),
                OsStr::new("--flag=@value")
            ]
        );
    }

    #[test]
    fn test_expand_not_exist() {
        let err = expand_params(["foo", "@this_file_does_not_exist"]).unwrap_err();
        assert!(
            err.to_string()
                .contains("error processing parameter \"@this_file_does_not_exist\""),
            "err={err:?}"
        );
    }

    #[test]
    fn test_expand_recursive() {
        let tempdir = tempfile::tempdir().unwrap();
        let param_file_a = tempdir.path().join("a");
        let param_file_b = tempdir.path().join("b");
        std::fs::write(
            &param_file_a,
            format!("@{}", param_file_b.to_str().unwrap()),
        )
        .unwrap();
        std::fs::write(&param_file_b, "bar\nbaz\n").unwrap();

        assert_eq!(
            expand_params(["foo", &format!("@{}", param_file_a.to_str().unwrap())]).unwrap(),
            ["foo", "bar", "baz"]
        );
    }

    #[test]
    fn test_expand_recursive_loop() {
        let tempdir = tempfile::tempdir().unwrap();
        let param_file_a = tempdir.path().join("a");
        let param_file_b = tempdir.path().join("b");
        std::fs::write(
            &param_file_a,
            format!("@{}", param_file_b.to_str().unwrap()),
        )
        .unwrap();
        std::fs::write(
            &param_file_b,
            format!("@{}", param_file_a.to_str().unwrap()),
        )
        .unwrap();

        let err =
            expand_params(["foo", &format!("@{}", param_file_b.to_str().unwrap())]).unwrap_err();
        assert!(
            format!("{err:?}").contains("parameter file loop detected"),
            "err={err:?}"
        );
    }
}
