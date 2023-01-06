// Copyright 2022 The ChromiumOS Authors.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{anyhow, bail, Result};
use binarypackage::{OutputFileSpec, BinaryPackage, XpakSpec}
use clap::Parser;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use version::Version;

const EBUILD_EXT: &str = "ebuild";
const BINARY_EXT: &str = "tbz2";
const MAIN_SCRIPT: &str = "/mnt/host/bazel-build/build_package.sh";

#[derive(Debug, Clone)]
struct SysrootFileSpec {
    sysroot_path: PathBuf,
    src_path: PathBuf,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct Cli {
    #[arg(short, long, required = true)]
    board: String,

    #[arg(short, long, required = true)]
    ebuild: String,

    #[arg(short, long, required = true)]
    file: Vec<String>,

    #[arg(short, long, required = true)]
    distfile: Vec<String>,

    #[arg(short, long, required = true)]
    output: String,

    #[arg(
        short,
        long = "<XPAK key>=[?]<output file>: Write the XPAK key from the binpkg to the \
    specified file. If =? is used then an empty file is created if XPAK key doesn't exist.",
        required = true
    )]
    xpak: Vec<XpakSpec>,

    #[arg(
        short,
        long = "<inside path>=<outside path>: Extracts a file from the binpkg and writes it to the outside path",
        required = true
    )]
    output_file: Vec<OutputFileSpec>,

    #[arg(
        short,
        long = "<inside path>=<outside path>: Copies the outside file into the sysroot",
        required = true
    )]
    sysroot_file: Vec<SysrootFileSpec>,
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
    pub fn install<P: AsRef<Path>>(self: &Self, sysroot: P) -> Result<()> {
        // TODO: Maybe we can hard link or bindmount the files to save the copy cost?
        let dest = sysroot.as_ref().join(&self.sysroot_path);
        let dest_dir = dest
            .parent()
            .ok_or_else(|| anyhow!("{dest:?} must have a parent"))?;
        std::fs::create_dir_all(dest_dir)?;
        std::fs::copy(&self.src_path, dest)?;
        Ok(())
    }
}

fn extract_binary_package_files<P: AsRef<Path>>(
    bin_pkg: P,
    xpak_specs: &Vec<&XpakSpec>,
    output_file_specs: &Vec<&OutputFileSpec>,
) -> Result<()> {
    if xpak_specs.is_empty() && output_file_specs.is_empty() {
        return Ok(());
    }
    let mut pkg = BinaryPackage::new(bin_pkg)?;
    pkg.extract_xpak_files(xpak_specs)?;
    pkg.extract_out_files(output_file_specs)?;
    Ok(())
}

struct EbuildMetadata {
    overlay: String,
    category: String,
    package_name: String,
    version: version::Version,
}

impl EbuildMetadata {
    fn parse<P: AsRef<Path>>(path: P) -> Result<Self> {
        // ParseEbuildMetadata expects path to be in the following form:
        // <overlay>/<category>/<packageName>/<packageName>-<version>.ebuild
        // i.e., third_party/chromiumos-overlay/app-accessibility/brltty/brltty-6.3-r6.ebuild
        // TODO: this currently fails with absolute paths.
        let path_str = path.as_ref().to_string_lossy();
        let stripped = path_str.strip_suffix(EBUILD_EXT).ok_or(anyhow!(
            "ebuild must have .ebuild suffix (got {:?}",
            path.as_ref()
        ))?;
        let (rest, version) = Version::from_str_suffix(&stripped)?;
        let parts: Vec<_> = rest.split("/").collect();
        if parts.len() < 4 {
            bail!("unable to parse ebuild path: {:?}", path.as_ref())
        }

        return Ok(Self {
            version,
            package_name: parts[parts.len() - 2].to_string(),
            category: parts[parts.len() - 3].to_string(),
            overlay: parts[0..parts.len() - 3].join("/"),
        });
    }
}

fn main() {
    let r = runfiles::Runfiles::create();
    let runScript = r.rlocation("chromiumos/bazel/ebuild/private/cmd/build_package/build_package.sh");

    let _args = Cli::parse()?;
    let _cfg = mountsdk::Cli::parse()?;

//     cfg.BindMounts = append(cfg.BindMounts, makechroot.BindMount{
//         Source:    runScript,
//         MountPath: mainScript,
//     })
//
//     v := strings.SplitN(ebuildSource, "=", 2)
//     if len(v) < 2 {
//         return errors.New("invalid ebuild cfg")
//     }
//
//     ebuildMetadata, err := ParseEbuildMetadata(v[0])
//     if err != nil {
//         return fmt.Errorf("invalid ebuild file name: %w", err)
//     }
//
//     ebuildFile := makechroot.BindMount{
//         Source:    v[1],
//         MountPath: filepath.Join(mountsdk.SourceDir, "src", ebuildMetadata.Overlay, ebuildMetadata.Category, ebuildMetadata.PackageName, filepath.Base(ebuildSource)),
//     }
//     cfg.Remounts = append(cfg.Remounts, filepath.Dir(ebuildFile.MountPath))
//
//     cfg.BindMounts = append(cfg.BindMounts, ebuildFile)
//     for _, fileSpec := range fileSpecs {
//         v := strings.SplitN(fileSpec, "=", 2)
//         if len(v) < 2 {
//         return errors.New("invalid file cfg")
//         }
//         cfg.BindMounts = append(cfg.BindMounts, makechroot.BindMount{
//         Source:    v[1],
//         MountPath: filepath.Join(filepath.Dir(ebuildFile.MountPath), v[0]),
//     })
// }
//
//     for _, distfileSpec := range distfileSpecs {
//     v := strings.SplitN(distfileSpec, "=", 2)
//     if len(v) < 2 {
//     return errors.New("invalid distfile cfg")
//     }
//     cfg.BindMounts = append(cfg.BindMounts, makechroot.BindMount{
//     Source:    v[1],
//     MountPath: filepath.Join("/var/cache/distfiles", v[0]),
//     })
//     }
//
//     targetPackagesDir := filepath.Join("/build", board, "packages")
//
//     if err := mountsdk.RunInSDK(cfg, func(s *mountsdk.MountedSDK) error {
//     overlayEbuildPath := s.RootDir.Add(ebuildFile.MountPath)
//     for _, dir := range []string{targetPackagesDir, "/var/lib/portage/pkgs"} {
//     if err := os.MkdirAll(s.RootDir.Add(dir).Outside(), 0o755); err != nil {
//     return err
//     }
//     }
//
//     sysroot := s.RootDir.Add("build").Add(board).Outside()
//
//     if err := installSysrootFiles(sysrtootFileSpecs, sysroot); err != nil {
//     return err
//     }
//
//     cmd := s.Command(mainScript, "ebuild", "--skip-manifest", overlayEbuildPath.Inside(), "clean", "package")
//     cmd.Env = append(cmd.Env, fmt.Sprintf("BOARD=%s", board))
//
//     if err := processes.Run(ctx, cmd); err != nil {
//     return err
//     }
//
//     // TODO: Normalize timestamps in the archive.
//     binaryOutPath := filepath.Join(targetPackagesDir,
//     ebuildMetadata.Category,
//     strings.TrimSuffix(filepath.Base(ebuildSource), ebuildExt)+binaryExt)
//
//     if err := fileutil.Copy(filepath.Join(s.DiffDir, binaryOutPath), finalOutPath); err != nil {
//     return err
//     }
//
//     if err := extractBinaryPackageFiles(finalOutPath, xpakSpecs, outputFileSpecs); err != nil {
//     return err
//     }
//
//     return nil
//     }); err != nil {
//     if err, ok := err.(*exec.ExitError); ok {
//     return cliutil.ExitCode(err.ExitCode())
//     }
//     return err
//     }
//
//     return nil
// }
}
