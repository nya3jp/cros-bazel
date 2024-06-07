// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use rayon::prelude::*;
use std::{
    collections::BTreeMap,
    fs::{read_link, File},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use tempfile::tempdir;
use walkdir::WalkDir;

use crate::util::files_equal;

const MODE_MASK: u32 = 0o7777;

/// Contains all the DiffItems for each path that didn't match.
pub struct DiffAnalysis(DiffAnalysisInner);

type DiffAnalysisInner = BTreeMap<PathBuf, Vec<DiffItem>>;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum DiffItem {
    OnlyInLeft,
    OnlyInRight,
    /// Left and right have differing types (i.e., symlink, dir, file).
    TypeMismatch {
        // TODO: Add an enum so we can convey they types.
    },
    ModeMismatch {
        left_mode: u32,
        right_mode: u32,
    },
    SymlinkContentsMismatch {
        left_target: PathBuf,
        right_target: PathBuf,
    },
    FileContentsMismatch {
        diff: String,
    },
}

impl std::fmt::Display for DiffItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DiffItem::OnlyInLeft => write!(f, "Only in left"),
            DiffItem::OnlyInRight => write!(f, "Only in right"),
            DiffItem::TypeMismatch {} => write!(f, "Type mismatch"),
            DiffItem::ModeMismatch {
                left_mode,
                right_mode,
            } => write!(f, "Mode {left_mode:o} != {right_mode:o}"),
            DiffItem::SymlinkContentsMismatch {
                left_target,
                right_target,
            } => write!(f, "Symlink target {left_target:?} != {right_target:?}"),
            DiffItem::FileContentsMismatch { diff } => write!(f, "{}", diff),
        }
    }
}

impl DiffAnalysis {
    pub fn display(&self) -> DiffAnalysisPrinter {
        DiffAnalysisPrinter { analysis: self }
    }
}
pub struct DiffAnalysisPrinter<'a> {
    analysis: &'a DiffAnalysis,
}

impl<'a> IntoIterator for &'a DiffAnalysis {
    type Item = <&'a DiffAnalysisInner as IntoIterator>::Item;
    type IntoIter = <&'a DiffAnalysisInner as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for DiffAnalysis {
    type Item = <DiffAnalysisInner as IntoIterator>::Item;
    type IntoIter = <DiffAnalysisInner as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl std::fmt::Display for DiffAnalysisPrinter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (path, items) in self.analysis {
            for item in items {
                match item {
                    DiffItem::FileContentsMismatch { diff } => writeln!(f, "{}", diff)?,
                    item => writeln!(
                        f,
                        "{}: {}",
                        if path == Path::new("") {
                            Path::new(".").display()
                        } else {
                            path.display()
                        },
                        item
                    )?,
                };
            }
        }

        Ok(())
    }
}

fn invoke_diff(left: &Path, right: &Path, common: &Path) -> Result<String> {
    let mut cmd = std::process::Command::new("diff");
    cmd.arg("-u").arg("--no-dereference").arg(left).arg(right);

    let output = cmd.output().context("Failed to find `diff`")?;

    let stderr = String::from_utf8(output.stderr).context("Invalid stderr")?;

    // diff will return 0 only if the files match (which they won't).
    if !output.status.success() && !stderr.is_empty() {
        bail!("Error while invoking: {:?}\n{:?}", cmd, stderr);
    }

    let mut lines = vec![];
    for (i, raw_line) in output.stdout.split(|c| *c == b'\n').enumerate() {
        if i > 300 {
            lines.push("* Truncated diff (too long) *".into());
            break;
        }
        match String::from_utf8(raw_line.into()) {
            Ok(line) => lines.push(line),
            Err(_) => {
                lines.push("* Truncated diff (non UTF-8 found in diff) *".into());
                break;
            }
        }
    }

    // Rewrite the diff header to remove timestamps and absolute paths
    if lines.len() > 2 && lines[0].starts_with("--- ") && lines[1].starts_with("+++ ") {
        lines[0] = format!("--- a/{}", common.display());
        lines[1] = format!("+++ b/{}", common.display());
    };

    // Drop trailing newlines
    while let Some(last) = lines.last() {
        if last.is_empty() {
            lines.pop();
        } else {
            break;
        }
    }

    Ok(lines.into_iter().join("\n"))
}

fn dump_elf(src: &Path, dest: &Path) -> Result<()> {
    let mut cmd = std::process::Command::new("readelf");
    // Dump everything except relocations since they are very large.
    cmd.args(["-h", "-l", "-S", "-s", "-d", "-V", "-A", "-I"]);
    cmd.arg(src);

    let output = cmd
        .stdout(File::create(dest).with_context(|| format!("open {dest:?}"))?)
        .output()
        .context("Failed to find `readelf`")?;

    let stdout = String::from_utf8(output.stdout).context("Invalid stdout")?;
    let stderr = String::from_utf8(output.stderr).context("Invalid stderr")?;

    if !output.status.success() {
        bail!("Failed while invoking: {:?}\n{:?}{:?}", cmd, stdout, stderr);
    }

    Ok(())
}

fn diff_elf(left: &Path, right: &Path, common: &Path) -> Result<String> {
    let tmpdir = tempdir()?;

    let left_dump = &tmpdir.path().join("left.elf");
    dump_elf(left, left_dump).with_context(|| format!("Error dumping ELF: {left:?}"))?;

    let right_dump = &tmpdir.path().join("right.elf");
    dump_elf(right, right_dump).with_context(|| format!("Error dumping ELF: {right:?}"))?;

    invoke_diff(left_dump, right_dump, common)
}

fn diff_files(left: &Path, right: &Path, common: &Path) -> Result<String> {
    let left_type =
        infer::get_from_path(left).with_context(|| format!("inferring type for: {left:?}"))?;
    let right_type =
        infer::get_from_path(right).with_context(|| format!("inferring type for: {right:?}"))?;

    let (Some(left_type), Some(right_type)) = (left_type, right_type) else {
        return invoke_diff(left, right, common);
    };

    if left_type.mime_type() != right_type.mime_type() {
        return Ok(format!(
            "Mime types differ: {} != {}",
            left_type.mime_type(),
            right_type.mime_type(),
        ));
    }

    let mime_type = left_type.mime_type();

    // TODO: Add zip and tarball diffs
    if mime_type == "application/x-executable" {
        return diff_elf(left, right, common);
    }

    invoke_diff(left, right, common)
}

fn collect_entries(path: &Path) -> Result<BTreeMap<PathBuf, std::fs::Metadata>> {
    let mut entries = BTreeMap::new();

    for entry in WalkDir::new(path) {
        let entry = entry?;

        let metadata = entry
            .metadata()
            .with_context(|| format!("stat {:?}", entry.path()))?;

        let relative_path = entry.into_path().strip_prefix(path).unwrap().into();

        entries.insert(relative_path, metadata);
    }

    Ok(entries)
}

/// Diffs the two file paths and returns the analysis.
///
/// Only diffs will be returned. An empty set means the two paths are identical.
pub fn diff(left_root: &Path, right_root: &Path) -> Result<DiffAnalysis> {
    let mut missing_results = BTreeMap::new();

    // Traverse the trees in parallel.
    let mut entries = [left_root, right_root]
        .into_par_iter()
        .map(|root| collect_entries(root).with_context(|| format!("Failed traversing {root:?}")))
        .collect::<Result<Vec<_>>>()?;

    let mut left_entries = entries.remove(0);
    let mut right_entries = entries.remove(0);

    left_entries.retain(|path, _| {
        if right_entries.contains_key(path) {
            true
        } else {
            missing_results.insert(path.to_path_buf(), vec![DiffItem::OnlyInLeft]);
            false
        }
    });

    right_entries.retain(|path, _| {
        if left_entries.contains_key(path) {
            true
        } else {
            missing_results.insert(path.to_path_buf(), vec![DiffItem::OnlyInRight]);
            false
        }
    });

    let mut merged_metadata = vec![];
    // left_entries and right_entries contain identical keys now.
    for (path, left_metadata) in left_entries {
        let right_metadata = right_entries.remove(&path).unwrap();
        merged_metadata.push((path, left_metadata, right_metadata));
    }

    let mut results: BTreeMap<_, _> = merged_metadata
        .into_par_iter()
        .map(|(path, left_metadata, right_metadata)| {
            let mut items = vec![];

            let left = left_root.join(&path);
            let right = right_root.join(&path);

            let left_mode = left_metadata.permissions().mode() & MODE_MASK;
            let right_mode = right_metadata.permissions().mode() & MODE_MASK;

            if left_metadata.is_dir() && right_metadata.is_dir() {
                // TODO: Compare xattrs
                if left_mode != right_mode {
                    items.push(DiffItem::ModeMismatch {
                        left_mode,
                        right_mode,
                    });
                }
            } else if left_metadata.is_symlink() && right_metadata.is_symlink() {
                let left_target = read_link(&left).with_context(|| format!("readlink {left:?}"))?;
                let right_target =
                    read_link(&right).with_context(|| format!("readlink {right:?}"))?;

                // TODO: Compare xattrs
                if left_target != right_target {
                    items.push(DiffItem::SymlinkContentsMismatch {
                        left_target,
                        right_target,
                    });
                }
            } else if left_metadata.is_file() && right_metadata.is_file() {
                // TODO: Comare xattrs
                if left_mode != right_mode {
                    items.push(DiffItem::ModeMismatch {
                        left_mode,
                        right_mode,
                    });
                }

                if !files_equal(&left, &right)? {
                    let diff = diff_files(&left, &right, &path)?;

                    items.push(DiffItem::FileContentsMismatch { diff });
                }
            } else {
                items.push(DiffItem::TypeMismatch {});
            }

            Ok((path, items))
        })
        .filter(|result| match result {
            Ok((_path, items)) => !items.is_empty(),
            Err(_) => true,
        })
        .collect::<Result<_>>()?;

    // I wish there was a .collect_into() so we could avoid this step.
    results.extend(missing_results);

    Ok(DiffAnalysis(results))
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, create_dir_all},
        io::Write,
        os::unix::fs::symlink,
    };

    use anyhow::bail;
    use tempfile::tempdir;

    use super::*;

    fn testdata(path: impl AsRef<Path>) -> Result<PathBuf> {
        let r = runfiles::Runfiles::create()?;
        let path = runfiles::rlocation!(
            r,
            Path::new("cros/bazel/portage/bin/xpaktool/testdata").join(path.as_ref())
        );

        if !path.try_exists()? {
            bail!("{:?} was not found", path);
        }

        Ok(path)
    }

    #[test]
    fn empty_dir() -> Result<()> {
        let tmpdir = tempdir()?;
        let tmpdir = tmpdir.path();

        let a = tmpdir.join("a");
        create_dir_all(&a)?;

        let b = tmpdir.join("b");
        create_dir_all(&b)?;

        let diff = diff(&a, &b)?;
        assert_eq!(diff.0, BTreeMap::from([]));

        assert_eq!(diff.display().to_string(), "");

        Ok(())
    }

    #[test]
    fn dir_permission_mismatch() -> Result<()> {
        let tmpdir = tempdir()?;
        let tmpdir = tmpdir.path();

        let a = tmpdir.join("a");
        create_dir_all(&a)?;
        std::fs::set_permissions(&a, std::fs::Permissions::from_mode(0o755))?;

        let b = tmpdir.join("b");
        create_dir_all(&b)?;
        std::fs::set_permissions(&b, std::fs::Permissions::from_mode(0o750))?;

        let diff = diff(&a, &b)?;
        assert_eq!(
            diff.0,
            BTreeMap::from([(
                "".into(),
                vec![DiffItem::ModeMismatch {
                    left_mode: 0o755,
                    right_mode: 0o750
                },]
            )])
        );

        assert_eq!(diff.display().to_string(), ".: Mode 755 != 750\n");

        Ok(())
    }

    #[test]
    fn only_in_left() -> Result<()> {
        let tmpdir = tempdir()?;
        let tmpdir = tmpdir.path();

        let a = tmpdir.join("a");
        create_dir_all(a.join("only"))?;
        fs::write(a.join("file"), "")?;

        let b = tmpdir.join("b");
        create_dir_all(&b)?;

        let diff = diff(&a, &b)?;
        assert_eq!(
            diff.0,
            BTreeMap::from([
                ("only".into(), vec![DiffItem::OnlyInLeft]),
                ("file".into(), vec![DiffItem::OnlyInLeft]),
            ])
        );

        assert_eq!(
            diff.display().to_string(),
            "file: Only in left\nonly: Only in left\n"
        );

        Ok(())
    }

    #[test]
    fn only_in_right() -> Result<()> {
        let tmpdir = tempdir()?;
        let tmpdir = tmpdir.path();

        let a = tmpdir.join("a");
        create_dir_all(&a)?;

        let b = tmpdir.join("b");
        create_dir_all(&b.join("only"))?;
        fs::write(b.join("file"), "")?;

        let diff = diff(&a, &b)?;
        assert_eq!(
            diff.0,
            BTreeMap::from([
                ("only".into(), vec![DiffItem::OnlyInRight]),
                ("file".into(), vec![DiffItem::OnlyInRight]),
            ])
        );

        assert_eq!(
            diff.display().to_string(),
            "file: Only in right\nonly: Only in right\n"
        );

        Ok(())
    }

    #[test]
    fn symlinks() -> Result<()> {
        let tmpdir = tempdir()?;
        let tmpdir = tmpdir.path();

        let a = tmpdir.join("a");
        create_dir_all(&a)?;
        symlink("hello", a.join("same"))?;
        symlink("world", a.join("diff"))?;

        let b = tmpdir.join("b");
        create_dir_all(&b)?;
        symlink("hello", b.join("same"))?;
        symlink("bar", b.join("diff"))?;

        let diff = diff(&a, &b)?;
        assert_eq!(
            diff.0,
            BTreeMap::from([(
                "diff".into(),
                vec![DiffItem::SymlinkContentsMismatch {
                    left_target: "world".into(),
                    right_target: "bar".into(),
                }],
            ),])
        );

        assert_eq!(
            diff.display().to_string(),
            "diff: Symlink target \"world\" != \"bar\"\n"
        );

        Ok(())
    }

    #[test]
    fn type_mismatch() -> Result<()> {
        let tmpdir = tempdir()?;
        let tmpdir = tmpdir.path();

        let a = tmpdir.join("a");
        create_dir_all(&a)?;
        symlink("world", a.join("hello"))?;

        let b = tmpdir.join("b");
        create_dir_all(&b)?;
        fs::write(b.join("hello"), "world")?;

        let diff = diff(&a, &b)?;
        assert_eq!(
            diff.0,
            BTreeMap::from([("hello".into(), vec![DiffItem::TypeMismatch {}]),])
        );

        assert_eq!(diff.display().to_string(), "hello: Type mismatch\n");

        Ok(())
    }

    #[test]
    fn files_equal() -> Result<()> {
        let tmpdir = tempdir()?;
        let tmpdir = tmpdir.path();

        let a = tmpdir.join("a");
        create_dir_all(&a)?;
        fs::write(a.join("hello"), "world")?;

        let b = tmpdir.join("b");
        create_dir_all(&b)?;
        fs::write(b.join("hello"), "world")?;

        let diff = diff(&a, &b)?;
        assert_eq!(diff.0, BTreeMap::from([]));

        assert_eq!(diff.display().to_string(), "");

        Ok(())
    }

    #[test]
    fn files_not_equal_plain_text() -> Result<()> {
        let tmpdir = tempdir()?;
        let tmpdir = tmpdir.path();

        let a = tmpdir.join("a");
        create_dir_all(&a)?;
        fs::write(a.join("hello"), "world")?;
        std::fs::set_permissions(a.join("hello"), std::fs::Permissions::from_mode(0o644))?;

        let b = tmpdir.join("b");
        create_dir_all(&b)?;
        fs::write(b.join("hello"), "world!")?;
        std::fs::set_permissions(b.join("hello"), std::fs::Permissions::from_mode(0o640))?;

        let diff = diff(&a, &b)?;
        assert_eq!(
            diff.0,
            BTreeMap::from([(
                "hello".into(),
                vec![
                    DiffItem::ModeMismatch {
                        left_mode: 0o644,
                        right_mode: 0o640
                    },
                    DiffItem::FileContentsMismatch {
                        diff: r#"--- a/hello
+++ b/hello
@@ -1 +1 @@
-world
\ No newline at end of file
+world!
\ No newline at end of file"#
                            .into(),
                    }
                ]
            ),])
        );

        assert_eq!(
            diff.display().to_string(),
            r#"hello: Mode 644 != 640
--- a/hello
+++ b/hello
@@ -1 +1 @@
-world
\ No newline at end of file
+world!
\ No newline at end of file
"#
        );

        Ok(())
    }

    #[test]
    fn files_not_equal_elf() -> Result<()> {
        let tmpdir = tempdir()?;
        let tmpdir = tmpdir.path();

        let a = tmpdir.join("a");
        create_dir_all(&a)?;
        std::fs::copy(testdata("simple_lib.so")?, a.join("lib.so"))?;

        let b = tmpdir.join("b");
        create_dir_all(&b)?;
        std::fs::copy(testdata("simple_versioned_lib.so")?, b.join("lib.so"))?;

        // Manually implement the match since we only want to check part of the
        // diff to verify it came from readelf.
        for entry in diff(&a, &b)? {
            match entry {
                (path, items) if path == Path::new("lib.so") && items.len() == 1 => {
                    match &items[0] {
                        DiffItem::FileContentsMismatch { diff } => {
                            assert!(
                                diff.contains("Entry point address"),
                                "Expected readelf output, got: {}",
                                diff
                            );
                        }
                        _ => bail!("Unexpected item: {:?}", &items[0]),
                    }
                }
                _ => bail!("Unexpected entry: {entry:?}"),
            }
        }

        Ok(())
    }

    #[test]
    fn files_not_equal_shellball() -> Result<()> {
        let tmpdir = tempdir()?;
        let tmpdir = tmpdir.path();

        let a = tmpdir.join("a");
        create_dir_all(&a)?;
        let mut log = File::create(a.join("log.txt"))?;
        // Create a file with ASCII at the beginning binary data at the end.
        // i.e., a shellball.
        // The ASCII part needs to be large enough to trick diff that this is
        // a diffable file.
        for i in 0..1050 {
            if i == 10 {
                log.write_all(b"abcd efg\n")?;
            } else if i < 1000 {
                log.write_all(b"abcdefg\n")?;
            } else {
                log.write_all(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0xce, 0x28])?;
            }
        }

        let b = tmpdir.join("b");
        create_dir_all(&b)?;
        let mut log = File::create(b.join("log.txt"))?;
        for i in 0..1050 {
            if i < 1000 {
                log.write_all(b"abcdefg\n")?;
            } else {
                log.write_all(&[9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0xce, 0x28])?;
            }
        }

        let diff = diff(&a, &b)?;
        assert_eq!(
            diff.0,
            BTreeMap::from([(
                "log.txt".into(),
                vec![DiffItem::FileContentsMismatch {
                    diff: r#"--- a/log.txt
+++ b/log.txt
@@ -8,7 +8,6 @@
 abcdefg
 abcdefg
 abcdefg
-abcd efg
 abcdefg
 abcdefg
 abcdefg
@@ -998,4 +997,5 @@
 abcdefg
 abcdefg
 abcdefg
* Truncated diff (non UTF-8 found in diff) *"#
                        .into(),
                }]
            ),])
        );

        Ok(())
    }

    #[test]
    fn files_not_equal_binary_blob() -> Result<()> {
        let tmpdir = tempdir()?;
        let tmpdir = tmpdir.path();

        let a = tmpdir.join("a");
        create_dir_all(&a)?;
        let mut log = File::create(a.join("log.txt"))?;
        for _ in 0..100 {
            log.write_all(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 0])?;
        }

        let b = tmpdir.join("b");
        create_dir_all(&b)?;
        let mut log = File::create(b.join("log.txt"))?;
        for _ in 0..100 {
            log.write_all(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])?;
        }

        // Manually implement the match since we only want to check part of the
        // diff to verify it came from readelf.
        for entry in diff(&a, &b)? {
            match entry {
                (path, items) if path == Path::new("log.txt") && items.len() == 1 => {
                    match &items[0] {
                        DiffItem::FileContentsMismatch { diff } => {
                            assert!(
                                diff.contains("Binary files ") && diff.contains(" differ"),
                                "Binary files X and X differ, got: '{}'",
                                diff
                            );
                        }
                        _ => bail!("Unexpected item: {:?}", &items[0]),
                    }
                }
                _ => bail!("Unexpected entry: {entry:?}"),
            }
        }

        Ok(())
    }
}
