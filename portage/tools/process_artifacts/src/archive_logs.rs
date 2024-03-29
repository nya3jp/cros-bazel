// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::HashMap,
    io::{Seek, SeekFrom, Write},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, ensure, Result};

use crate::proto::build_event_stream::{
    BuildEvent, BuildEventId, BuildEventPayload, File, NamedSetOfFiles, NamedSetOfFilesId,
};

struct FileSetIndex {
    index: HashMap<NamedSetOfFilesId, NamedSetOfFiles>,
}

impl<'a, T> From<T> for FileSetIndex
where
    T: IntoIterator<Item = &'a BuildEvent>,
{
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

impl FileSetIndex {
    pub fn files(&self, id: &NamedSetOfFilesId) -> Result<Vec<File>> {
        let mut files: Vec<File> = Vec::new();
        self.collect_files(id, &mut files)?;
        Ok(files)
    }

    fn collect_files(&self, id: &NamedSetOfFilesId, files: &mut Vec<File>) -> Result<()> {
        let Some(entry) = self.index.get(id) else {
            bail!("NamedSetOfFiles {} not found", id.id);
        };
        files.extend(entry.files.clone());
        for subset_id in &entry.file_sets {
            self.collect_files(subset_id, files)?;
        }
        Ok(())
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
    let index: FileSetIndex = events.into();

    let mut paths: Vec<PathBuf> = Vec::new();

    for event in events {
        let BuildEventId::TargetCompleted(complete_id) = &event.id else {
            continue;
        };
        let BuildEventPayload::Completed(complete) = &event.payload else {
            bail!(
                "Corrupted BuildEvent: TargetCompleted for {}: payload is something else",
                complete_id.label
            );
        };
        for output_group in &complete.output_group {
            for file_set_id in &output_group.file_sets {
                for file in index.files(file_set_id)? {
                    paths.push(path_for_file(&file));
                }
            }
        }
    }

    paths.sort();
    paths.dedup();

    let mut input_file = tempfile::tempfile()?;
    for path in paths {
        input_file.write_all(path.as_os_str().as_bytes())?;
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

    use crate::load_build_events_jsonl;

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
}
