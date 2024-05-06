// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use itertools::Itertools;

use crate::{processors::build_event::BuildEventProcessor, proto::ebuild_metadata::EbuildMetadata};

const PREBUILT_HEADER: &str = r#"# Provides the CAS location for all ebuild binary packages
# used by the build. This is useful when modifying primordial code such as
# build_packges, or some of the eclasses.
#
# Place this file at `$WORKSPACE/prebuilts.bzl` (i.e., ~/chromiumos/src/prebuilts.bzl),
# then pass `--config=prebuilts` to your `bazel build` command.
"#;

fn load_metadata(path: &Path) -> Result<EbuildMetadata> {
    let file = std::fs::File::open(path).with_context(|| format!("Failed to open {path:?}"))?;

    let metadata: EbuildMetadata =
        serde_json::from_reader(file).with_context(|| format!("Failed parsing {path:?}"))?;

    Ok(metadata)
}

pub fn compute_prebuilts(
    output_path: &Path,
    workspace_dir: &Path,
    processor: &BuildEventProcessor,
) -> Result<()> {
    let mut output =
        std::fs::File::create(output_path).with_context(|| format!("path: {output_path:?}"))?;

    let Some(remote_instance_name) = processor.get_command_flag("remote_instance_name") else {
        write!(output, "# --remote_instance_name was not set.")?;
        return Ok(());
    };

    write!(output, "{PREBUILT_HEADER}")?;

    let metadata_files = processor.get_output_group_files("ebuild_metadata")?;

    let metadata = metadata_files
        .into_iter()
        .map(|relative_path| workspace_dir.join(relative_path))
        .filter_map(|path| match load_metadata(&path) {
            Ok(metadata) => Some(Ok(metadata)),
            Err(e) => {
                // Bazel currently has a bug where some output group files will be missing.
                // We just ignore these failures for now.
                // https://github.com/bazelbuild/bazel/issues/20737
                eprintln!("Failed to load: {path:?}: {e}");
                match writeln!(output, "# Failed to load: {path:?}: {e}") {
                    Ok(_) => None,
                    Err(e) => Some(Err(e.into())),
                }
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let args = metadata
        .iter()
        .sorted_by_key(|metadata| &metadata.label)
        .map(|metadata| {
            format!(
                "--{}_prebuilt=cas://{}/{}/{}",
                metadata.label, remote_instance_name, metadata.sha256, metadata.size
            )
        })
        .collect_vec();

    let contents = if args.is_empty() {
        String::new()
    } else {
        let mut contents = std::iter::once("build:prebuilts")
            .chain(args.iter().map(String::as_str))
            .join(" \\\n ");
        contents.push('\n');

        contents
    };

    write!(output, "{contents}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::load_build_events_jsonl;
    use runfiles::Runfiles;
    use std::path::PathBuf;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn basic() -> Result<()> {
        let testdata_path = PathBuf::from("cros/bazel/portage/tools/process_artifacts/testdata");

        let r = Runfiles::create()?;
        let events = load_build_events_jsonl(&runfiles::rlocation!(
            r,
            testdata_path.join("ebuild_metadata.bep.jsonl")
        ))?;

        let processor = BuildEventProcessor::from(&events);

        // Set up a workspace directory containing fake artifacts.
        let workspace_dir = tempfile::TempDir::new()?;
        let workspace_dir = workspace_dir.path();
        for (test_path, relative_path) in [
            (
                "linux-headers.metadata.json",
                "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/target/host/chromiumos/sys-kernel/linux-headers/4.14-r92_metadata.json"
            ), (
                "glibc.metadata.json",
                "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/target/host/chromiumos/sys-libs/glibc/2.37-r7_metadata.json"
            ), (
                "os-headers.metadata.json",
                "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/target/host/portage-stable/virtual/os-headers/0-r2_metadata.json"
            ),
        ] {
            let path = workspace_dir.join(relative_path);
            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::copy(&runfiles::rlocation!(r, testdata_path.join(test_path)), path)?;
        }

        let tmpdir = tempdir()?;
        let output_path = tmpdir.path().join("prebuilts.bzl");

        // Create an archive.
        compute_prebuilts(&output_path, workspace_dir, &processor)?;

        let output = std::fs::read_to_string(&output_path)?;

        assert_eq!(
            output,
            format!(
                "{}{}",
                PREBUILT_HEADER,
                r#"build:prebuilts \
 --@@_main~portage~portage//internal/packages/stage1/target/host/chromiumos/sys-kernel/linux-headers:4.14-r92_prebuilt=cas://projects/chromeos-bot/instances/cros-rbe-nonrelease/c38045633a8cec7411aa0618e63896b52b0234ddd7136a34a69ef2af9134b74d/1322061 \
 --@@_main~portage~portage//internal/packages/stage1/target/host/chromiumos/sys-libs/glibc:2.37-r7_prebuilt=cas://projects/chromeos-bot/instances/cros-rbe-nonrelease/3cd79108475a96af2874f9fbd602093748c3d60d4e7e2265c2f55dc82fa8a21c/21002060 \
 --@@_main~portage~portage//internal/packages/stage1/target/host/portage-stable/virtual/os-headers:0-r2_prebuilt=cas://projects/chromeos-bot/instances/cros-rbe-nonrelease/a623fd5b2aca004886fb9a1ff070d999101e07b4353535e889eca0612c2c7d56/14851
"#
            )
        );

        Ok(())
    }

    #[test]
    fn missing_metadata() -> Result<()> {
        let testdata_path = PathBuf::from("cros/bazel/portage/tools/process_artifacts/testdata");

        let r = Runfiles::create()?;
        let events = load_build_events_jsonl(&runfiles::rlocation!(
            r,
            testdata_path.join("ebuild_metadata.bep.jsonl")
        ))?;

        let processor = BuildEventProcessor::from(&events);

        // Set up a workspace directory containing fake artifacts.
        let workspace_dir = tempfile::TempDir::new()?;
        let workspace_dir = workspace_dir.path();
        for (test_path, relative_path) in [
            (
                "linux-headers.metadata.json",
                "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/target/host/chromiumos/sys-kernel/linux-headers/4.14-r92_metadata.json"
            ), (
                "glibc.metadata.json",
                "bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/target/host/chromiumos/sys-libs/glibc/2.37-r7_metadata.json"
            ),
        ] {
            let path = workspace_dir.join(relative_path);
            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::copy(&runfiles::rlocation!(r, testdata_path.join(test_path)), path)?;
        }

        let tmpdir = tempdir()?;
        let output_path = tmpdir.path().join("prebuilts.bzl");

        // Create an archive.
        compute_prebuilts(&output_path, workspace_dir, &processor)?;

        let output = std::fs::read_to_string(&output_path)?;

        assert_eq!(
            output,
            format!(
                r#"{}# Failed to load: "{}/bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/target/host/portage-stable/virtual/os-headers/0-r2_metadata.json": Failed to open "{}/bazel-out/k8-fastbuild/bin/external/_main~portage~portage/internal/packages/stage1/target/host/portage-stable/virtual/os-headers/0-r2_metadata.json"
{}"#,
                PREBUILT_HEADER,
                workspace_dir.display(),
                workspace_dir.display(),
                r#"build:prebuilts \
 --@@_main~portage~portage//internal/packages/stage1/target/host/chromiumos/sys-kernel/linux-headers:4.14-r92_prebuilt=cas://projects/chromeos-bot/instances/cros-rbe-nonrelease/c38045633a8cec7411aa0618e63896b52b0234ddd7136a34a69ef2af9134b74d/1322061 \
 --@@_main~portage~portage//internal/packages/stage1/target/host/chromiumos/sys-libs/glibc:2.37-r7_prebuilt=cas://projects/chromeos-bot/instances/cros-rbe-nonrelease/3cd79108475a96af2874f9fbd602093748c3d60d4e7e2265c2f55dc82fa8a21c/21002060
"#
            )
        );

        Ok(())
    }
}
