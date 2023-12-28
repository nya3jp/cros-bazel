// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use either::Either;
use std::{
    ffi::{OsStr, OsString},
    fs,
    os::unix::ffi::OsStrExt,
    path::Path,
};

fn os_str_strip_prefix<'a>(s: &'a OsStr, prefix: &OsStr) -> Option<&'a OsStr> {
    if s.as_bytes().starts_with(prefix.as_bytes()) {
        Some(OsStr::from_bytes(&s.as_bytes()[prefix.as_bytes().len()..]))
    } else {
        None
    }
}

fn read_param_file<P: AsRef<Path>>(path: P, file_prefix_symbol: &str) -> Result<Vec<OsString>> {
    let contents = fs::read_to_string(path)?;
    contents
        .lines()
        .enumerate()
        .map(|(lineno, line)| {
            if line.starts_with(file_prefix_symbol) {
                bail!("line {lineno}: recursive parameter file expansion is not supported");
            }
            Ok(OsString::from(line))
        })
        .collect()
}

fn expand_single_param(param: OsString, prefix: &str) -> Result<impl Iterator<Item = OsString>> {
    match os_str_strip_prefix(&param, OsStr::from_bytes(prefix.as_bytes())) {
        Some(path) => Ok(Either::Left(
            read_param_file(path, prefix)
                .with_context(|| format!("error processing parameter {:?}", param))?
                .into_iter(),
        )),
        None => Ok(Either::Right(std::iter::once(param.into()))),
    }
}

/// Expand command line arguments containing parameter files, denoted by
/// an argument starting with the character `@`.
///
/// For an argument `@file`, the arguments read from the file are inserted in
/// place of the original @file argument.
/// Arguments in the parameter file are terminated by newlines.
///
/// Nested parameter file expansion is unsupported.
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

    for item in itr {
        let param: OsString = item.into();
        expanded.extend(expand_single_param(param, "@")?);
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
        let param_file = tempdir.path().join("param_file");
        std::fs::write(&param_file, "@recursion\n").unwrap();

        let err = expand_params([[OsStr::new("@"), param_file.as_os_str()].join(OsStr::new(""))])
            .unwrap_err();
        assert!(
            format!("{err:?}").contains("recursive parameter file expansion is not supported"),
            "err={err:?}"
        );
        assert!(
            format!("{err:?}").contains(param_file.to_str().unwrap()),
            "err={err:?} does not contain the problematic parameter file path"
        );
    }
}
