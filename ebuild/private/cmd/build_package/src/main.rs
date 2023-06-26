// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{anyhow, bail, ensure, Context, Result};
use clap::{command, Parser};
use cliutil::cli_main;
use container::{enter_mount_namespace, BindMount, CommonArgs, ContainerSettings};
use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf},
    process::ExitCode,
    str::FromStr,
};

const EBUILD_EXT: &str = ".ebuild";
const MAIN_SCRIPT: &str = "/mnt/host/.build_package/build_package.sh";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,

    /// Name of board
    #[arg(long)]
    board: Option<String>,

    #[arg(long, required = true)]
    ebuild: EbuildMetadata,

    #[arg(long)]
    file: Vec<BindMount>,

    #[arg(long)]
    distfile: Vec<BindMount>,

    #[arg(long, help = "Git trees used by CROS_WORKON_TREE")]
    git_tree: Vec<PathBuf>,

    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(
        long,
        help = "<inside path>=<outside path>: Copies the outside file into the sysroot"
    )]
    sysroot_file: Vec<SysrootFileSpec>,

    #[arg(long, help = "Allows network access during build")]
    allow_network_access: bool,

    #[arg(long)]
    test: bool,
}

#[derive(Debug, Clone)]
struct SysrootFileSpec {
    sysroot_path: PathBuf,
    src_path: PathBuf,
}

impl FromStr for SysrootFileSpec {
    type Err = anyhow::Error;
    fn from_str(spec: &str) -> Result<Self> {
        let (sysroot_path, src_path) = cliutil::split_key_value(spec)?;
        let sysroot_path = PathBuf::from(sysroot_path);
        if !sysroot_path.is_absolute() {
            bail!(
                "Invalid sysroot spec: {:?}, {:?} must be absolute",
                spec,
                sysroot_path
            )
        }
        Ok(Self {
            sysroot_path,
            src_path: PathBuf::from(src_path),
        })
    }
}

impl SysrootFileSpec {
    pub fn install(&self, sysroot: &Path) -> Result<()> {
        // TODO: Maybe we can hard link or bindmount the files to save the copy cost?
        let dest = sysroot.join(&self.sysroot_path);
        let dest_dir = dest
            .parent()
            .with_context(|| format!("{dest:?} must have a parent"))?;
        std::fs::create_dir_all(dest_dir)?;
        std::fs::copy(&self.src_path, dest)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct EbuildMetadata {
    source: PathBuf,
    mount_path: PathBuf,
    category: String,
    _package_name: String,
    file_name: String,
}

impl FromStr for EbuildMetadata {
    type Err = anyhow::Error;

    fn from_str(spec: &str) -> Result<Self> {
        let (path, source) = cliutil::split_key_value(spec)?;
        // We expect path to be in the following form:
        // <category>/<packageName>/<packageName>-<version>.ebuild
        // i.e., third_party/chromiumos-overlay/app-accessibility/brltty/brltty-6.3-r6.ebuild
        let parts: Vec<_> = path.split('/').collect();
        if parts.len() < 3 {
            bail!("unable to parse ebuild path: {:?}", path)
        }

        Ok(Self {
            source: source.into(),
            mount_path: path.into(),
            category: parts[parts.len() - 3].into(),
            _package_name: parts[parts.len() - 2].into(),
            file_name: parts[parts.len() - 1].into(),
        })
    }
}

fn do_main() -> Result<()> {
    let args = Cli::parse();

    let mut settings = ContainerSettings::new();
    settings.apply_common_args(&args.common)?;

    let runfiles = runfiles::Runfiles::create()?;

    settings.push_bind_mount(BindMount {
        source: runfiles.rlocation("cros/bazel/ebuild/private/cmd/build_package/build_package.sh"),
        mount_path: PathBuf::from(MAIN_SCRIPT),
        rw: false,
    });

    settings.push_bind_mount(BindMount {
        source: args.ebuild.source,
        mount_path: args.ebuild.mount_path.clone(),
        rw: false,
    });

    let ebuild_mount_dir = args.ebuild.mount_path.parent().unwrap();

    for mount in args.file {
        settings.push_bind_mount(BindMount {
            source: mount.source,
            mount_path: ebuild_mount_dir.join(mount.mount_path),
            rw: false,
        })
    }

    for mount in args.distfile {
        settings.push_bind_mount(BindMount {
            source: mount.source,
            mount_path: PathBuf::from("/var/cache/distfiles").join(mount.mount_path),
            rw: false,
        })
    }

    let mut seen_git_trees = HashSet::with_capacity(args.git_tree.len());

    for file in &args.git_tree {
        // Either <SHA> or <SHA>.tar.xxx
        let tree_file = file.file_name();

        if !seen_git_trees.insert(tree_file) {
            bail!("Duplicate git tree {:?} specified.", tree_file);
        }

        settings.push_bind_mount(BindMount {
            source: file.to_path_buf(),
            mount_path: PathBuf::from("/var/cache/trees")
                .join(file.file_name().expect("path to contain file name")),
            rw: false,
        })
    }

    settings.set_allow_network_access(args.allow_network_access);

    let (portage_tmp_dir, portage_pkg_dir) = match &args.board {
        Some(board) => {
            let root_dir = Path::new("/build").join(board);
            (root_dir.join("tmp/portage"), root_dir.join("packages"))
        }
        None => (
            PathBuf::from("/var/tmp/portage"),
            PathBuf::from("/var/lib/portage/pkgs"),
        ),
    };

    let mut container = settings.prepare()?;

    let upper_dir = container.upper_dir().to_owned();

    let out_dir = upper_dir.join(portage_pkg_dir.strip_prefix("/")?);
    std::fs::create_dir_all(out_dir)?;

    // HACK: CrOS disables pkg_pretend in emerge(1), but ebuild(1) still tries to
    // run it.
    // TODO(b/280233260): Remove this hack once we fix ebuild(1).
    let pretend_stamp_path = upper_dir
        .join(portage_tmp_dir.strip_prefix("/")?)
        .join(&args.ebuild.category)
        .join(
            args.ebuild
                .file_name
                .strip_suffix(EBUILD_EXT)
                .with_context(|| anyhow!("Ebuild file must end with .ebuild"))?,
        )
        .join(".pretended");
    std::fs::create_dir_all(pretend_stamp_path.parent().unwrap())?;
    File::create(pretend_stamp_path)?;

    let sysroot = match &args.board {
        Some(board) => upper_dir.join("build").join(board),
        None => upper_dir.to_owned(),
    };
    for spec in args.sysroot_file {
        spec.install(&sysroot)?;
    }

    let mut command = container.command(MAIN_SCRIPT);
    command
        .arg("ebuild")
        .arg("--skip-manifest")
        .arg(args.ebuild.mount_path)
        .arg("package");
    if args.test {
        command.arg("test");
    }
    if let Some(board) = args.board {
        command.env("BOARD", board);
    }

    let status = command.status()?;
    ensure!(status.success());

    let binary_out_path = portage_pkg_dir.join(args.ebuild.category).join(format!(
        "{}.tbz2",
        args.ebuild
            .file_name
            .strip_suffix(EBUILD_EXT)
            .with_context(|| anyhow!("Ebuild file must end with .ebuild"))?
    ));

    if let Some(output) = args.output {
        std::fs::copy(upper_dir.join(binary_out_path.strip_prefix("/")?), output)
            .with_context(|| format!("{binary_out_path:?} wasn't produced by build_package"))?;
    }

    Ok(())
}

fn main() -> ExitCode {
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cli_main(do_main)
}
