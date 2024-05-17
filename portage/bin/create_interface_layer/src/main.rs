// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::bail;
use anyhow::{ensure, Context, Result};
use clap::{command, Parser};
use cliutil::{cli_main, expanded_args_os};
use container::{enter_mount_namespace, BindMount, CommonArgs, ContainerSettings};
use durabletree::DurableTree;
use fileutil::{remove_dir_all_with_chmod, with_permissions, SafeTempDirBuilder};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use std::io::Write;
use walkdir::WalkDir;

use std::collections::{BTreeMap, BTreeSet};
use std::format;
use std::fs::{read_link, File};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::{os::unix::process::ExitStatusExt, path::PathBuf, process::ExitCode};

const INPUT: &str = "/.input";
const WORK_LIST: &str = "/.work";
const OUTPUT: &str = "/.output";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,

    // Specifies the sysroot inside the `input` layer to operate on.
    #[arg(long)]
    sysroot: PathBuf,

    // Input layer that contains the staged contents of the package.
    #[arg(long)]
    input: PathBuf,

    // Output directory where the contents will be saved as a durable tree.
    #[arg(long)]
    output: PathBuf,
}

/// Returns true if `path` is inside the `sysroot` and in one of the `dirs`.
fn directory_matches(sysroot: impl AsRef<Path>, dirs: &[&str], path: impl AsRef<Path>) -> bool {
    let Ok(sysroot_relative_path) = path.as_ref().strip_prefix(sysroot.as_ref()) else {
        return false;
    };

    dirs.iter()
        .any(|exclude_dir| sysroot_relative_path.starts_with(exclude_dir))
}

fn is_excluded_directory(sysroot: impl AsRef<Path>, dir: impl AsRef<Path>) -> bool {
    directory_matches(
        sysroot,
        &[
            "usr/share/doc",
            "usr/share/info",
            "usr/share/man",
            // We shouldn't be executing any non-host binaries.
            "bin",
            "sbin",
            "usr/bin",
            "usr/sbin",
            "usr/local/bin",
            "usr/local/sbin",
            // We don't want any debug symbols since they contain line numbers.
            "usr/lib/debug",
        ],
        dir,
    )
}

fn is_toolchain_lib(sysroot: impl AsRef<Path>, path: impl AsRef<Path>) -> bool {
    directory_matches(sysroot, &["usr/lib64/clang"], path)
}

fn is_rust_registry(sysroot: impl AsRef<Path>, path: impl AsRef<Path>) -> bool {
    directory_matches(sysroot, &["usr/lib/cros_rust_registry"], path)
}

fn in_library_path(sysroot: impl AsRef<Path>, path: impl AsRef<Path>) -> bool {
    directory_matches(sysroot, &["usr/lib", "usr/lib64"], path)
}

fn is_build_bin(sysroot: impl AsRef<Path>, path: impl AsRef<Path>) -> bool {
    directory_matches(sysroot, &["build/bin"], path)
}

/// All paths are relative to root.
struct WorkItems {
    directories_to_create: BTreeSet<PathBuf>,
    files_to_copy: BTreeMap<PathBuf, std::fs::FileType>,
    interface_libraries: BTreeSet<PathBuf>,
}

fn traverse_input(root: &Path, sysroot: &Path) -> Result<WorkItems> {
    let mut directories_to_create: BTreeSet<PathBuf> = BTreeSet::new();
    let mut files_to_copy: BTreeMap<PathBuf, std::fs::FileType> = BTreeMap::new();
    let mut interface_libraries: BTreeSet<PathBuf> = BTreeSet::new();

    let is_so = Regex::new(r"\.so(\.[0-9]+)*$")?;
    let is_config = Regex::new(r"-config$")?;

    for entry in WalkDir::new(root).into_iter().filter_entry(|e| {
        !e.file_type().is_dir()
            || !is_excluded_directory(sysroot, e.path().strip_prefix(root).unwrap())
    }) {
        let entry = entry?;
        let relative_path = entry.path().strip_prefix(root)?;

        if entry.file_type().is_dir() {
            if !directories_to_create.contains(relative_path) {
                directories_to_create.insert(relative_path.to_path_buf());
            }
        } else {
            let file_name_str = relative_path
                .file_name()
                .expect("file name to exist")
                .to_str()
                .with_context(|| {
                    format!("{:?} is not a valid string", relative_path.file_name())
                })?;

            if in_library_path(sysroot, relative_path) {
                // We don't want to modify or omit any toolchain libraries
                // since they are needed when compiling.
                //
                // We also don't want to modify the rust crate registry since cargo
                // validates the checksums
                if is_toolchain_lib(sysroot, relative_path)
                    || is_rust_registry(sysroot, relative_path)
                {
                    files_to_copy.insert(relative_path.to_path_buf(), entry.file_type());
                } else if relative_path.extension().unwrap_or_default() == "a" {
                    // Drop all static libs.
                    continue;
                } else if entry.file_type().is_file() && is_so.is_match(file_name_str) {
                    interface_libraries.insert(relative_path.to_path_buf());
                } else {
                    files_to_copy.insert(relative_path.to_path_buf(), entry.file_type());
                }
            } else if is_build_bin(sysroot, relative_path) && is_config.is_match(file_name_str) {
                files_to_copy.insert(relative_path.to_path_buf(), entry.file_type());

                // The ${CHOST}-config scripts in /build/bin call into the scripts in
                // /usr/bin. Since we excluded `/usr/bin` above, we manually add the
                // files if they exist.
                let usr_bin = sysroot.join("usr/bin");
                let relative_config_path = usr_bin.join(file_name_str);
                if let Ok(config_metadata) = root.join(&relative_config_path).symlink_metadata() {
                    files_to_copy.insert(relative_config_path, config_metadata.file_type());
                    directories_to_create.insert(usr_bin);
                }
            } else {
                // Copy other support files.
                files_to_copy.insert(relative_path.to_path_buf(), entry.file_type());
            }

            if let Some(parent) = relative_path.parent() {
                if !directories_to_create.contains(parent) {
                    directories_to_create.insert(parent.to_path_buf());
                }
            }
        }
    }

    Ok(WorkItems {
        directories_to_create,
        files_to_copy,
        interface_libraries,
    })
}

fn copy_xattrs(src: &Path, dest: &Path) -> Result<()> {
    for key in xattr::list(src).with_context(|| format!("list xattrs on {src:?}"))? {
        if key.to_string_lossy().starts_with("user.") {
            let value = xattr::get(src, &key)
                .with_context(|| format!("get xattr {key:?} on {src:?}"))?
                .unwrap_or(vec![]);
            xattr::set(dest, &key, &value)
                .with_context(|| format!("set xattr {key:?} on {dest:?}"))?;
        }
    }
    Ok(())
}

fn copy_metadata(src: &Path, dest: &Path) -> Result<()> {
    copy_xattrs(src, dest)?;

    let metadata = src
        .symlink_metadata()
        .with_context(|| format!("stat {src:?}"))?;

    std::fs::set_permissions(dest, PermissionsExt::from_mode(metadata.mode()))
        .with_context(|| format!("chmod {:o} {:?}", metadata.mode(), &dest))?;

    Ok(())
}

fn create_empty_directories(dest_root: &Path, dirs: &BTreeSet<PathBuf>) -> Result<()> {
    for dir in dirs {
        let dest = dest_root.join(dir);

        std::fs::create_dir(&dest).with_context(|| format!("mkdir {dest:?}"))?;
    }

    Ok(())
}

fn copy_files(
    src_root: &Path,
    dest_root: &Path,
    files: &BTreeMap<PathBuf, std::fs::FileType>,
) -> Result<()> {
    files
        .par_iter()
        .try_for_each(|(file, file_type)| -> Result<()> {
            let src = src_root.join(file);
            let dest = dest_root.join(file);

            if file_type.is_symlink() {
                let original = read_link(&src).with_context(|| format!("readlink {src:?}"))?;
                std::os::unix::fs::symlink(&original, &dest)
                    .with_context(|| format!("ln -s {original:?} {dest:?}"))?;
                // symlinks don't have permissions and xattrs are not supported on
                // symlinks in user namespaces.
            } else if file_type.is_file() {
                // The std::fs::copy also copies permissions.
                std::fs::copy(&src, &dest).with_context(|| format!("cp {src:?} {dest:?}"))?;
                // Ensure we have +w on the file so we can set xattrs.
                with_permissions(&dest, 0o644, || copy_xattrs(&src, &dest))?;
            } else {
                bail!("{src:?} has an unknown file type: {file_type:?}");
            }

            Ok(())
        })?;

    Ok(())
}

fn create_interface_libraries(
    mutable_base_dir: &Path,
    args: &CommonArgs,
    src_root: &Path,
    dest_root: &Path,
    libraries: &BTreeSet<PathBuf>,
) -> Result<()> {
    // We use the SDK container to invoke llvm-ifs.
    let mut sdk = ContainerSettings::new();
    sdk.set_mutable_base_dir(mutable_base_dir);
    sdk.apply_common_args(args)?;

    sdk.push_bind_mount(BindMount {
        mount_path: INPUT.into(),
        source: src_root.into(),
        rw: false,
    });

    sdk.push_bind_mount(BindMount {
        mount_path: OUTPUT.into(),
        source: dest_root.into(),
        rw: true,
    });

    let work_list_path = mutable_base_dir.join("libraries");
    let mut work_list = File::create(&work_list_path)?;
    libraries
        .iter()
        .try_for_each(|path| write!(work_list, "{}\0", path.display()))?;

    sdk.push_bind_mount(BindMount {
        mount_path: WORK_LIST.into(),
        source: work_list_path,
        rw: false,
    });

    let mut container = sdk.prepare()?;

    // Use xargs to run in parallel.
    let mut command = container.command("xargs");
    command.current_dir(INPUT);
    command
        .arg("--null")
        .arg("--arg-file")
        .arg(WORK_LIST)
        .arg("--max-procs")
        .arg("0")
        .arg("--no-run-if-empty")
        .arg("-I{}");
    // Generate interface libraries
    command
        .arg("llvm-ifs")
        .arg("--input-format=ELF")
        .arg("--output-elf")
        .arg(Path::new(OUTPUT).join("{}"))
        .arg("--output-ifs")
        .arg(Path::new(OUTPUT).join("{}.ifs"))
        .arg("{}");

    let status = command.status()?;
    ensure!(
        status.success(),
        "Command failed: status={:?}, code={:?}, signal={:?}",
        status,
        status.code(),
        status.signal()
    );

    for lib in libraries {
        copy_metadata(&src_root.join(lib), &dest_root.join(lib))?;
    }

    Ok(())
}

fn finalize_directory_permissions(
    src_root: &Path,
    dest_root: &Path,
    dirs: &BTreeSet<PathBuf>,
) -> Result<()> {
    // Iterate in reverse so we set the permissions for the deepest directory
    // first.
    for dir in dirs.iter().rev() {
        copy_metadata(&src_root.join(dir), &dest_root.join(dir))?;
    }

    Ok(())
}

fn do_main() -> Result<()> {
    let args = Cli::try_parse_from(expanded_args_os()?)?;

    let sysroot = args.sysroot.strip_prefix("/").with_context(|| {
        format!(
            "Expected sysroot to be absolute, got {}",
            args.sysroot.display()
        )
    })?;

    remove_dir_all_with_chmod(&args.output)
        .with_context(|| format!("rm -rf {:?}", &args.output))?;

    let mutable_base_dir = SafeTempDirBuilder::new()
        .base_dir(args.output.parent().context("output missing parent")?)
        .build()?;

    let mut input = ContainerSettings::new();
    input.set_mutable_base_dir(mutable_base_dir.path());
    input.push_layer(&args.input)?;
    let input = input.mount()?;

    let work = traverse_input(input.path(), sysroot)?;

    create_empty_directories(&args.output, &work.directories_to_create)?;

    copy_files(input.path(), &args.output, &work.files_to_copy)?;

    create_interface_libraries(
        mutable_base_dir.path(),
        &args.common,
        input.path(),
        &args.output,
        &work.interface_libraries,
    )?;

    finalize_directory_permissions(input.path(), &args.output, &work.directories_to_create)?;

    DurableTree::convert(&args.output)
}

fn main() -> ExitCode {
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cli_main(do_main, Default::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_excluded_directory() -> Result<()> {
        assert!(is_excluded_directory("", "usr/bin"));
        assert!(!is_excluded_directory("", "usr/share"));
        assert!(!is_excluded_directory("build/board", "usr/bin"));
        assert!(!is_excluded_directory("build/board", "usr/share"));
        assert!(is_excluded_directory("build/board", "build/board/usr/bin"));
        assert!(!is_excluded_directory(
            "build/board",
            "build/board/usr/share"
        ));

        Ok(())
    }
}
