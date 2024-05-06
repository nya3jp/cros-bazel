// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    io::{Seek, SeekFrom, Write},
    os::unix::ffi::OsStrExt,
    path::Path,
    process::Command,
};

use anyhow::{ensure, Result};

use crate::processors::build_event::BuildEventProcessor;

pub fn archive_logs(
    output_path: &Path,
    workspace_dir: &Path,
    processor: &BuildEventProcessor,
) -> Result<()> {
    let files = processor.get_output_group_files("transitive_logs")?;

    let mut input_file = tempfile::tempfile()?;
    for relative_path in files {
        // Ignore non-existent files.
        if workspace_dir.join(&relative_path).try_exists()? {
            input_file.write_all(relative_path.as_os_str().as_bytes())?;
            input_file.write_all(&[0])?;
        }
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
        proto::build_event_stream::{
            BuildEvent, BuildEventId, BuildEventPayload, File, NamedSetOfFiles, NamedSetOfFilesId,
            OutputGroup, TargetComplete, TargetCompletedId,
        },
    };

    use super::*;

    #[test]
    fn basic() -> Result<()> {
        let r = Runfiles::create()?;
        let events = load_build_events_jsonl(&runfiles::rlocation!(
            r,
            "cros/bazel/portage/tools/process_artifacts/testdata/bep.jsonl"
        ))?;

        let processor = BuildEventProcessor::from(&events);

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
            // We create *.tbz2, but they should not be included in the tarball.
            "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/\
             target/host/chromiumos/sys-kernel/linux-headers/linux-headers-4.14-r92.tbz2",
            // We omit artifacts for virtual/os-headers.
        ] {
            let path = workspace_dir.join(relative_path);
            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::write(path, relative_path)?;
        }

        let output_path = tempfile::Builder::new().suffix(".tar.gz").tempfile()?;
        let output_path = output_path.path();

        // Create an archive.
        archive_logs(output_path, workspace_dir, &processor)?;

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

        let events: Vec<BuildEvent> = vec![];
        let processor = BuildEventProcessor::from(&events);

        archive_logs(output_path, workspace_dir, &processor)?;

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

        let processor = BuildEventProcessor::from(&events);

        let workspace_dir = tempfile::TempDir::new()?;
        let workspace_dir = workspace_dir.path();

        // Create a.txt, but not b.txt, so that `archive_logs` would fail if it attempts to archive
        // the other output group.
        std::fs::write(workspace_dir.join("a.txt"), [])?;

        let output_path = tempfile::Builder::new().suffix(".tar.gz").tempfile()?;
        let output_path = output_path.path();

        archive_logs(output_path, workspace_dir, &processor)?;

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

        let processor = BuildEventProcessor::from(&events);

        let workspace_dir = tempfile::TempDir::new()?;
        let workspace_dir = workspace_dir.path();
        let output_path = tempfile::Builder::new().suffix(".tar.gz").tempfile()?;
        let output_path = output_path.path();

        archive_logs(output_path, workspace_dir, &processor)?;

        Ok(())
    }
}
