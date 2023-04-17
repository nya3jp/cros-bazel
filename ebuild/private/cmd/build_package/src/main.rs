// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{anyhow, bail, Context, Result};
use clap::{command, Parser};
use cliutil::cli_main;
use makechroot::BindMount;
use mountsdk::{ConfigArgs, MountedSDK};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::ExitCode,
    str::FromStr,
};
use version::Version;

const EBUILD_EXT: &str = ".ebuild";
const MAIN_SCRIPT: &str = "/mnt/host/bazel-build/build_package.sh";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    #[command(flatten)]
    mountsdk_config: ConfigArgs,

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
    overlay: String,
    category: String,
    package_name: String,
    file_name: String,
}

impl FromStr for EbuildMetadata {
    type Err = anyhow::Error;

    fn from_str(spec: &str) -> Result<Self> {
        let (path, source) = cliutil::split_key_value(spec)?;
        // We expect path to be in the following form:
        // <overlay>/<category>/<packageName>/<packageName>-<version>.ebuild
        // i.e., third_party/chromiumos-overlay/app-accessibility/brltty/brltty-6.3-r6.ebuild
        // TODO: this currently fails with absolute paths.
        let stripped = path
            .strip_suffix(EBUILD_EXT)
            .ok_or_else(|| anyhow!("ebuild must have .ebuild suffix (got {:?}", path))?;
        let (rest, _) = Version::from_str_suffix(stripped)?;
        let parts: Vec<_> = rest.split('/').collect();
        if parts.len() < 4 {
            bail!("unable to parse ebuild path: {:?}", path)
        }

        return Ok(Self {
            source: source.into(),
            package_name: parts[parts.len() - 2].into(),
            category: parts[parts.len() - 3].into(),
            overlay: parts[0..parts.len() - 3].join("/"),
            file_name: Path::new(source)
                .file_name()
                .with_context(|| "Ebuild must have a file name")?
                .to_string_lossy()
                .into(),
        });
    }
}

fn do_main() -> Result<()> {
    let args = Cli::parse();
    let runfiles_mode = args.mountsdk_config.runfiles_mode();
    let mut cfg = mountsdk::Config::try_from(args.mountsdk_config)?;

    let r = runfiles::Runfiles::create()?;

    let fix_runfile_path = |path| {
        if runfiles_mode {
            r.rlocation(path)
        } else {
            path
        }
    };

    cfg.bind_mounts.push(BindMount {
        source: r.rlocation("cros/bazel/ebuild/private/cmd/build_package/build_package.sh"),
        mount_path: PathBuf::from(MAIN_SCRIPT),
    });

    let ebuild_mount_dir: PathBuf = [
        mountsdk::SOURCE_DIR,
        "src",
        &args.ebuild.overlay,
        &args.ebuild.category,
        &args.ebuild.package_name,
    ]
    .iter()
    .collect();
    let ebuild_path = ebuild_mount_dir.join(&args.ebuild.file_name);
    cfg.bind_mounts.push(BindMount {
        source: fix_runfile_path(args.ebuild.source),
        mount_path: ebuild_path.clone(),
    });

    for mount in args.file {
        cfg.bind_mounts.push(BindMount {
            source: fix_runfile_path(mount.source),
            mount_path: ebuild_mount_dir.join(mount.mount_path),
        })
    }

    for mount in args.distfile {
        cfg.bind_mounts.push(BindMount {
            source: fix_runfile_path(mount.source),
            mount_path: PathBuf::from("/var/cache/distfiles").join(mount.mount_path),
        })
    }

    let mut seen_git_trees = HashSet::with_capacity(args.git_tree.len());

    for file in &args.git_tree {
        // Either <SHA> or <SHA>.tar.xxx
        let tree_file = file.file_name();

        if !seen_git_trees.insert(tree_file) {
            bail!("Duplicate git tree {:?} specified.", tree_file);
        }

        cfg.bind_mounts.push(BindMount {
            source: file.to_path_buf(),
            mount_path: PathBuf::from("/var/cache/trees")
                .join(file.file_name().expect("path to contain file name")),
        })
    }

    if args.allow_network_access {
        cfg.allow_network_access = true;
    }

    let target_packages_dir: PathBuf = ["/build", &cfg.board, "packages"].iter().collect();

    let mut sdk = MountedSDK::new(cfg)?;
    let out_dir = sdk
        .root_dir()
        .outside
        .join(target_packages_dir.strip_prefix("/")?);
    std::fs::create_dir_all(out_dir)?;
    std::fs::create_dir_all(sdk.root_dir().outside.join("var/lib/portage/pkgs"))?;

    let sysroot = sdk.root_dir().join("build").join(&sdk.board).outside;
    for spec in args.sysroot_file {
        spec.install(&sysroot)?;
    }
    let ebuild_path_str = ebuild_path.to_string_lossy();
    let mut cmd_args = vec![
        MAIN_SCRIPT,
        "ebuild",
        "--skip-manifest",
        &ebuild_path_str,
        "clean",
        "package",
    ];
    if args.test {
        cmd_args.push("test");
    }
    sdk.run_cmd(&cmd_args)?;

    let binary_out_path = target_packages_dir.join(args.ebuild.category).join(format!(
        "{}.tbz2",
        args.ebuild
            .file_name
            .strip_suffix(EBUILD_EXT)
            .with_context(|| anyhow!("Ebuild file must end with .ebuild"))?
    ));

    if let Some(output) = args.output {
        std::fs::copy(
            sdk.diff_dir().join(binary_out_path.strip_prefix("/")?),
            output,
        )
        .with_context(|| format!("{binary_out_path:?} wasn't produced by build_package"))?;
    }

    Ok(())
}

fn main() -> ExitCode {
    cli_main(do_main)
}
