// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::BTreeMap,
    io::{Seek, SeekFrom, Write},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, ensure, Result};
use itertools::Itertools;

use crate::proto::build_event_stream::{
    BuildEvent, BuildEventId, BuildEventPayload, File, NamedSetOfFiles, NamedSetOfFilesId,
};

/// Provides fast merge operations over [`NamedSetOfFiles`].
///
/// [`NamedSetOfFiles`] can be deeply nested with duplications (like diamond inheritance).
/// Therefore, on keeping track of files, it is important to also keep track of named set IDs that
/// are already merged to the set, in order to avoid repeatedly merging the same named sets.
#[derive(Clone, Default)]
struct FastFileSet {
    // Invariant: named sets are closed over subsets, i.e. all subsets of the named sets are also in
    // the named sets.
    children: BTreeMap<NamedSetOfFilesId, NamedSetOfFiles>,
}

impl FastFileSet {
    /// Creates a new empty set.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns an iterator of files included in this set.
    pub fn files(&self) -> impl Iterator<Item = &File> {
        self.children
            .values()
            .flat_map(|named_set| named_set.files.iter())
            .sorted()
            .dedup()
    }
}

/// An index of [`BuildEvent`] that provides fast file set operations.
struct FastFileSetIndex {
    index: BTreeMap<NamedSetOfFilesId, NamedSetOfFiles>,
}

impl<'a, T> From<T> for FastFileSetIndex
where
    T: IntoIterator<Item = &'a BuildEvent>,
{
    /// Constructs a new [`FastFileSetIndex`] from an iterator of [`BuildEvent`].
    fn from(into_iter: T) -> Self {
        let index = into_iter
            .into_iter()
            .filter_map(|event| {
                let BuildEventId::NamedSet(id) = &event.id else {
                    return None;
                };
                let BuildEventPayload::NamedSetOfFiles(named_set) = &event.payload else {
                    return None;
                };
                Some((id.clone(), named_set.clone()))
            })
            .collect();
        Self { index }
    }
}

impl FastFileSetIndex {
    /// Merges a named set of files to [`FastFileSet`].
    pub fn merge(&self, mut fs: FastFileSet, id: &NamedSetOfFilesId) -> Result<FastFileSet> {
        if fs.children.contains_key(id) {
            return Ok(fs);
        }
        let Some(entry) = self.index.get(id) else {
            bail!("NamedSetOfFiles {} not found", id.id);
        };
        fs.children.insert(id.clone(), entry.clone());
        for subset in &entry.file_sets {
            fs = self.merge(fs, subset)?;
        }
        Ok(fs)
    }
}

fn path_for_file(file: &File) -> PathBuf {
    let mut path = PathBuf::new();
    for prefix in &file.path_prefix {
        path.push(prefix);
    }
    path.push(&file.name);
    path
}

pub fn archive_logs(output_path: &Path, workspace_dir: &Path, events: &[BuildEvent]) -> Result<()> {
    let index: FastFileSetIndex = events.into();

    let mut fileset = FastFileSet::new();

    for event in events {
        let BuildEventId::TargetCompleted(_) = &event.id else {
            continue;
        };
        let BuildEventPayload::Completed(complete) = &event.payload else {
            // This can happen when the target was incomplete due to build errors.
            continue;
        };
        for output_group in &complete.output_group {
            if output_group.name != "transitive_logs" {
                continue;
            }
            for file_set_id in &output_group.file_sets {
                fileset = index.merge(fileset, file_set_id)?;
            }
        }
    }

    let mut input_file = tempfile::tempfile()?;
    for file in fileset.files() {
        input_file.write_all(path_for_file(file).as_os_str().as_bytes())?;
        input_file.write_all(&[0])?;
    }
    input_file.seek(SeekFrom::Start(0))?;

    let status = Command::new("tar")
        .arg("--create")
        .arg("--auto-compress")
        .arg("--file")
        .arg(output_path)
        // --directory is order-sensitive. Place it between --file and
        // --files-from.
        .arg("--directory")
        .arg(workspace_dir)
        .arg("--verbatim-files-from")
        .arg("--null")
        .arg("--files-from")
        .arg("/dev/stdin")
        .stdin(input_file)
        .status()?;

    ensure!(status.success(), "Failed to create tarball");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use anyhow::ensure;
    use runfiles::Runfiles;
    use walkdir::WalkDir;

    use crate::{
        load_build_events_jsonl,
        proto::build_event_stream::{OutputGroup, TargetComplete, TargetCompletedId},
    };

    use super::*;

    #[test]
    fn basic() -> Result<()> {
        let runfiles = Runfiles::create()?;
        let events = load_build_events_jsonl(
            &runfiles.rlocation("cros/bazel/portage/tools/process_artifacts/testdata/bep.jsonl"),
        )?;

        // Set up a workspace directory containing fake artifacts.
        let workspace_dir = tempfile::TempDir::new()?;
        let workspace_dir = workspace_dir.path();
        for relative_path in [
            "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/remote_toolchain_inputs.log",
            "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/remote_toolchain_inputs.profile.json",
            "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/sdk_from_archive.log",
            "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/sdk_from_archive.profile.json",
            "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/stage1.log",
            "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/stage1.profile.json",
            "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/\
             target/host/chromiumos/sys-kernel/linux-headers/linux-headers-4.14-r92.log",
            "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/\
             target/host/chromiumos/sys-kernel/linux-headers/linux-headers-4.14-r92.profile.json",
            "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/\
             target/host/portage-stable/virtual/os-headers/os-headers-0-r2.log",
            "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/\
             target/host/portage-stable/virtual/os-headers/os-headers-0-r2.profile.json",
            // We create *.tbz2, but they should not be included in the tarball.
            "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/\
             target/host/chromiumos/sys-kernel/linux-headers/linux-headers-4.14-r92.tbz2",
            "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/\
             target/host/portage-stable/virtual/os-headers/os-headers-0-r2.tbz2",
        ] {
            let path = workspace_dir.join(relative_path);
            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::write(path, relative_path)?;
        }

        let output_path = tempfile::Builder::new().suffix(".tar.gz").tempfile()?;
        let output_path = output_path.path();

        // Create an archive.
        archive_logs(output_path, workspace_dir, &events)?;

        // Extract a generated archive and inspect the contents.
        let extract_dir = tempfile::TempDir::new()?;
        let extract_dir = extract_dir.path();

        let status = Command::new("tar")
            .arg("--extract")
            .arg("--file")
            .arg(output_path)
            .arg("--directory")
            .arg(extract_dir)
            .arg("--verbose")
            .status()?;
        ensure!(status.success());

        let contents: Vec<String> = WalkDir::new(extract_dir)
            .sort_by_file_name()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| {
                entry
                    .path()
                    .strip_prefix(extract_dir)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();

        assert_eq!(
            contents,
            [
                "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/remote_toolchain_inputs.log",
                "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/remote_toolchain_inputs.profile.json",
                "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/sdk_from_archive.log",
                "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/sdk_from_archive.profile.json",
                "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/stage1.log",
                "bazel-out/k8-fastbuild/bin/bazel/portage/sdk/stage1.profile.json",
                "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/\
                 stage1/target/host/chromiumos/sys-kernel/linux-headers/linux-headers-4.14-r92.log",
                "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/\
                 stage1/target/host/chromiumos/sys-kernel/linux-headers/\
                 linux-headers-4.14-r92.profile.json",
                "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/\
                 stage1/target/host/portage-stable/virtual/os-headers/os-headers-0-r2.log",
                "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/\
                 stage1/target/host/portage-stable/virtual/os-headers/os-headers-0-r2.profile.json",
            ]
        );

        Ok(())
    }

    #[test]
    fn empty() -> Result<()> {
        let workspace_dir = tempfile::TempDir::new()?;
        let workspace_dir = workspace_dir.path();

        let output_path = tempfile::Builder::new().suffix(".tar.gz").tempfile()?;
        let output_path = output_path.path();

        archive_logs(output_path, workspace_dir, &[])?;

        Ok(())
    }

    #[test]
    fn output_group() -> Result<()> {
        let events = vec![
            BuildEvent {
                id: BuildEventId::NamedSet(NamedSetOfFilesId {
                    id: "a".to_string(),
                }),
                payload: BuildEventPayload::NamedSetOfFiles(NamedSetOfFiles {
                    files: vec![File {
                        name: "a.txt".to_string(),
                        path_prefix: vec![],
                    }],
                    file_sets: vec![],
                }),
            },
            BuildEvent {
                id: BuildEventId::NamedSet(NamedSetOfFilesId {
                    id: "b".to_string(),
                }),
                payload: BuildEventPayload::NamedSetOfFiles(NamedSetOfFiles {
                    files: vec![File {
                        name: "b.txt".to_string(),
                        path_prefix: vec![],
                    }],
                    file_sets: vec![],
                }),
            },
            BuildEvent {
                id: BuildEventId::TargetCompleted(TargetCompletedId {
                    label: "//foo".to_string(),
                    aspect: None,
                }),
                payload: BuildEventPayload::Completed(TargetComplete {
                    success: true,
                    output_group: vec![
                        OutputGroup {
                            name: "transitive_logs".to_string(),
                            file_sets: vec![NamedSetOfFilesId {
                                id: "a".to_string(),
                            }],
                            incomplete: false,
                        },
                        OutputGroup {
                            name: "other".to_string(),
                            file_sets: vec![NamedSetOfFilesId {
                                id: "b".to_string(),
                            }],
                            incomplete: false,
                        },
                    ],
                }),
            },
        ];

        let workspace_dir = tempfile::TempDir::new()?;
        let workspace_dir = workspace_dir.path();

        // Create a.txt, but not b.txt, so that `archive_logs` would fail if it attempts to archive
        // the other output group.
        std::fs::write(workspace_dir.join("a.txt"), [])?;

        let output_path = tempfile::Builder::new().suffix(".tar.gz").tempfile()?;
        let output_path = output_path.path();

        archive_logs(output_path, workspace_dir, &events)?;

        Ok(())
    }

    #[test]
    fn deeply_nested_named_sets() -> Result<()> {
        // Create a fibonacci-like structure of named sets.
        let mut events: Vec<BuildEvent> = vec![
            BuildEvent {
                id: BuildEventId::NamedSet(NamedSetOfFilesId {
                    id: "0".to_string(),
                }),
                payload: BuildEventPayload::NamedSetOfFiles(NamedSetOfFiles {
                    files: vec![],
                    file_sets: vec![],
                }),
            },
            BuildEvent {
                id: BuildEventId::NamedSet(NamedSetOfFilesId {
                    id: "1".to_string(),
                }),
                payload: BuildEventPayload::NamedSetOfFiles(NamedSetOfFiles {
                    files: vec![],
                    file_sets: vec![],
                }),
            },
        ];
        for i in 2..=100 {
            events.push(BuildEvent {
                id: BuildEventId::NamedSet(NamedSetOfFilesId { id: i.to_string() }),
                payload: BuildEventPayload::NamedSetOfFiles(NamedSetOfFiles {
                    files: vec![],
                    file_sets: vec![
                        NamedSetOfFilesId {
                            id: (i - 2).to_string(),
                        },
                        NamedSetOfFilesId {
                            id: (i - 1).to_string(),
                        },
                    ],
                }),
            })
        }
        events.push(BuildEvent {
            id: BuildEventId::TargetCompleted(TargetCompletedId {
                label: "//foo".to_string(),
                aspect: None,
            }),
            payload: BuildEventPayload::Completed(TargetComplete {
                output_group: vec![OutputGroup {
                    name: "transitive_logs".to_string(),
                    file_sets: vec![NamedSetOfFilesId {
                        id: "100".to_string(),
                    }],
                    incomplete: false,
                }],
                success: true,
            }),
        });

        let workspace_dir = tempfile::TempDir::new()?;
        let workspace_dir = workspace_dir.path();
        let output_path = tempfile::Builder::new().suffix(".tar.gz").tempfile()?;
        let output_path = output_path.path();

        archive_logs(output_path, workspace_dir, &events)?;

        Ok(())
    }
}
