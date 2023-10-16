// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::BufReader,
    io::Read,
    iter::Iterator,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use crate::package::is_default;

const EXECUTABLE_MASK: u32 = 0o111;
// All ELF files start with these 4 bytes.
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

static WRAP_ELF: &str = "cros/bazel/portage/bin/extract_package_from_manifest/package/wrap_elf";
static DEFAULT_INTERP: &str = "/lib64/ld-linux-x86-64.so.2";

fn is_default_interp(p: &Path) -> bool {
    p == Path::new(DEFAULT_INTERP)
}

fn default_interp() -> PathBuf {
    PathBuf::from(DEFAULT_INTERP)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ElfFileMetadata {
    #[serde(default = "default_interp", skip_serializing_if = "is_default_interp")]
    pub(crate) interp: PathBuf,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) libs: BTreeMap<String, PathBuf>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) rpath: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) runpath: Vec<PathBuf>,
}

/// Returns whether a path is a path to an elf binary.
fn is_elf_binary(p: &Path) -> Result<bool> {
    // While we've already filtered out shared libraries, we'll miss any not in LD_LIBRARY_PATH.
    let name = p
        .file_name()
        .context("Path must have file name")?
        .to_string_lossy();
    if name.ends_with(".so") || name.contains(".so.") {
        return Ok(false);
    }

    // Ensure symlinks don't get detected as elf files.
    let md = p.symlink_metadata()?;
    if !md.is_file() {
        return Ok(false);
    }
    if md.permissions().mode() & EXECUTABLE_MASK == 0 {
        return Ok(false);
    }
    let mut content_header: Vec<u8> = vec![];
    BufReader::new(std::fs::File::open(p)?.take(4)).read_to_end(&mut content_header)?;
    if content_header != ELF_MAGIC {
        return Ok(false);
    }

    Ok(true)
}

/// Wraps a tree of elf files
pub(crate) fn wrap_elf_files<'a>(
    root: &Path,
    ld_library_path: &[PathBuf],
    files: impl Iterator<Item = &'a Path>,
) -> Result<BTreeMap<PathBuf, ElfFileMetadata>> {
    let mut elf_binaries = files
        .map(|p| root.join(p.strip_prefix("/").unwrap()))
        .filter(|p| is_elf_binary(p).unwrap())
        .peekable();

    if elf_binaries.peek().is_none() {
        return Ok(Default::default());
    }
    let r = runfiles::Runfiles::create()?;
    let mut cmd = Command::new(r.rlocation(WRAP_ELF));
    cmd.arg("--sysroot")
        .arg(root)
        .arg("--ld-library-path")
        .args(ld_library_path)
        .arg("--elf-files")
        .args(elf_binaries);
    let out = cmd.output()?;
    let stdout = std::str::from_utf8(&out.stdout)?;
    let stderr = std::str::from_utf8(&out.stderr)?;
    if !out.status.success() {
        bail!("Failed to run {cmd:?}:\n\n{stdout}\n\n{stderr}");
    }
    // Print any warnings from the child process regardless of whether it was a failure or success.
    eprint!("{}", stderr);
    serde_json::from_str(stdout).with_context(|| format!("Failed to parse json from {stdout:?}"))
}
