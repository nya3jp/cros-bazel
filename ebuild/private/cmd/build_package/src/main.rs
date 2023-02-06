// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{anyhow, bail, Context, Result};
use clap::{command, Parser};
use makechroot::BindMount;
use mountsdk::{ConfigArgs, MountedSDK};
use std::{
    path::{Path, PathBuf},
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
    board: String,

    #[arg(long, required = true)]
    ebuild: EbuildMetadata,

    #[arg(long)]
    file: Vec<BindMount>,

    #[arg(long)]
    distfile: Vec<BindMount>,

    #[arg(long, required = true)]
    output: PathBuf,

    #[arg(
        long,
        help = "<inside path>=<outside path>: Copies the outside file into the sysroot"
    )]
    sysroot_file: Vec<SysrootFileSpec>,
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
        return Ok(Self {
            sysroot_path,
            src_path: PathBuf::from(src_path),
        });
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
            .ok_or(anyhow!("ebuild must have .ebuild suffix (got {:?}", path))?;
        let (rest, _) = Version::from_str_suffix(&stripped)?;
        let parts: Vec<_> = rest.split("/").collect();
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

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut cfg = mountsdk::Config::try_from(args.mountsdk_config)?;

    let r = runfiles::Runfiles::create()?;

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
        source: args.ebuild.source,
        mount_path: ebuild_path.clone(),
    });

    for mount in args.file {
        cfg.bind_mounts.push(BindMount {
            source: mount.source,
            mount_path: ebuild_mount_dir.join(mount.mount_path),
        })
    }

    for mount in args.distfile {
        cfg.bind_mounts.push(BindMount {
            source: mount.source,
            mount_path: PathBuf::from("/var/cache/distfiles").join(mount.mount_path),
        })
    }

    let target_packages_dir: PathBuf = ["/build", &args.board, "packages"].iter().collect();

    let mut sdk = MountedSDK::new(cfg)?;
    let out_dir = sdk
        .root_dir()
        .outside
        .join(target_packages_dir.strip_prefix("/")?);
    std::fs::create_dir_all(out_dir)?;
    std::fs::create_dir_all(sdk.root_dir().outside.join("var/lib/portage/pkgs"))?;

    let sysroot = sdk.root_dir().join("build").join(&args.board).outside;
    for spec in args.sysroot_file {
        spec.install(&sysroot)?;
    }
    let runfiles_dir = std::env::current_dir()?.join(r.rlocation(""));
    sdk.run_cmd(|cmd| {
        cmd.args([
            MAIN_SCRIPT,
            "ebuild",
            "--skip-manifest",
            &ebuild_path.to_string_lossy(),
            "clean",
            "package",
        ])
        .env("BOARD", args.board)
        .env("RUNFILES_DIR", runfiles_dir);
    })?;

    let binary_out_path = target_packages_dir.join(args.ebuild.category).join(format!(
        "{}.tbz2",
        args.ebuild
            .file_name
            .strip_suffix(EBUILD_EXT)
            .with_context(|| anyhow!("Ebuild file must end with .ebuild"))?
    ));
    std::fs::copy(
        sdk.diff_dir().join(binary_out_path.strip_prefix("/")?),
        &args.output,
    )
    .with_context(|| format!("{binary_out_path:?} wasn't produced by build_package"))?;

    Ok(())
}
