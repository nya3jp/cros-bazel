// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, ensure, Context, Error, Result};
use binarypackage::BinaryPackage;
use clap::Parser;
use cliutil::cli_main;
use container::{
    enter_mount_namespace, BindMount, CommonArgs, ContainerSettings, PreparedContainer,
};
use durabletree::DurableTree;
use fileutil::{resolve_symlink_forest, SafeTempDir, SafeTempDirBuilder};
use itertools::Itertools;
use nix::mount::{mount, umount2, MntFlags, MsFlags};
use runfiles::Runfiles;
use std::{
    fs::{remove_dir_all, File, Permissions},
    io::ErrorKind,
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    str::FromStr,
};
use tracing::info_span;
use vdb::{generate_vdb_contents, get_vdb_dir};

/// The directory name under the file system root where package files to be
/// installed to the target file system (aka "package image") are staged before
/// installation so that pkg_preinst can modify them.
/// This path must match with the directory prefix used on generating staged
/// contents layers.
const STAGE_DIR_NAME: &str = ".image";

/// Defines the format of the `--install` command line argument.
///
/// It is a comma-separated list of multiple file paths.
#[derive(Clone, Debug)]
struct InstallSpec {
    pub input_binary_package: PathBuf,
    pub input_installed_contents_dir: PathBuf,
    pub input_staged_contents_dir: PathBuf,
    pub output_preinst_dir: PathBuf,
    pub output_postinst_dir: PathBuf,
}

impl FromStr for InstallSpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (
            input_binary_package,
            input_installed_contents_dir,
            input_staged_contents_dir,
            output_preinst_dir,
            output_postinst_dir,
        ) = s
            .split(',')
            .collect_tuple()
            .context("--install must have 5 paths separated by commas")?;
        Ok(Self {
            input_binary_package: input_binary_package.into(),
            input_installed_contents_dir: input_installed_contents_dir.into(),
            input_staged_contents_dir: input_staged_contents_dir.into(),
            output_preinst_dir: output_preinst_dir.into(),
            output_postinst_dir: output_postinst_dir.into(),
        })
    }
}

/// Tracks a tmpfs mount point. It unmounts the file system on drop.
struct TmpfsTempDir {
    dir: SafeTempDir,
}

impl TmpfsTempDir {
    /// Creates a temporary directory and mounts a tmpfs on the directory.
    pub fn new() -> Result<TmpfsTempDir> {
        let dir = SafeTempDir::new()?;
        mount(
            Some("tmpfs"),
            dir.path(),
            Some("tmpfs"),
            MsFlags::empty(),
            Some("mode=0755"),
        )
        .context("Failed to mount tmpfs for extra dir")?;
        Ok(TmpfsTempDir { dir })
    }

    /// Returns the path of the tmpfs mount point.
    pub fn path(&self) -> &Path {
        self.dir.path()
    }
}

impl Drop for TmpfsTempDir {
    fn drop(&mut self) {
        umount2(self.dir.path(), MntFlags::MNT_DETACH)
            .with_context(|| format!("Failed to unmount {}", self.dir.path().display()))
            .unwrap();
    }
}

/// Moves a directory to another location, possibly by copying them across different file systems.
/// The target directory must not exist or empty initially.
fn move_directory(source_dir: &Path, target_dir: &Path) -> Result<()> {
    match std::fs::remove_dir(target_dir) {
        Err(e) if e.raw_os_error() == Some(libc::ENOTEMPTY) => {}
        other => other?,
    }

    // Use /bin/mv to perform cross-filesystem moves.
    let status = Command::new("/bin/mv")
        .arg("--")
        .arg(source_dir)
        .arg(target_dir)
        .status()?;
    ensure!(status.success(), "mv failed: {:?}", status);
    Ok(())
}

/// Checks if we can skip install hooks for the package.
fn can_skip_install_hooks(staged_contents_dir: &Path, cpf: &str, root_dir: &Path) -> Result<bool> {
    // TODO: Avoid assuming that the staged contents dir is a durable tree.
    let tree = DurableTree::expand(staged_contents_dir)?;

    for layer_dir in tree.layers().iter().rev() {
        let vdb_dir = get_vdb_dir(
            &layer_dir.join(
                root_dir
                    .strip_prefix("/")
                    .expect("--root-dir must be absolute"),
            ),
            cpf,
        );
        if let Ok(files) = std::fs::read_to_string(vdb_dir.join("CROS_BAZEL_HOOK_REQUIRES")) {
            if files.is_empty() {
                tracing::info!("Skipping install hooks safely");
                return Ok(true);
            }
            tracing::info!("We have to run install hooks because they access following files:");
            eprint!("{}", files);
            return Ok(false);
        }
    }

    bail!("CROS_BAZEL_HOOK_REQUIRES not found");
}

/// Bind-mounts the binary package to the container.
fn bind_mount_binary_package(
    settings: &mut ContainerSettings,
    binary_package: &Path,
    root_dir: &Path,
    category_pf: &str,
) -> Result<()> {
    let portage_pkg_dir = if root_dir == Path::new("/") {
        PathBuf::from("/var/lib/portage/pkgs")
    } else {
        root_dir.join("packages")
    };

    let binary_package_mount_path = portage_pkg_dir.join(format!("{}.tbz2", category_pf));
    let real_binary_package_path = resolve_symlink_forest(binary_package)?;

    settings.push_bind_mount(BindMount {
        mount_path: binary_package_mount_path.clone(),
        source: real_binary_package_path,
        rw: false,
    });

    Ok(())
}

/// Runs hook functions using `drive_binary_package.sh`.
fn run_hooks_general(
    container: &mut PreparedContainer,
    root_dir: &Path,
    category_pf: &str,
    phases: &[&str],
) -> Result<()> {
    let _span = info_span!("drive_binary_package").entered();

    // The temporary directory used by ebuilds. It's available to ebuilds as `$T`.
    // This must be under /tmp so that post-processing clears it from durable trees.
    const EBUILD_TEMP_DIR: &str = "/tmp/ebuild";

    let temp_dir = container.root_dir().join(
        EBUILD_TEMP_DIR
            .strip_prefix('/')
            .expect("EBUILD_TEMP_DIR must be absolute"),
    );
    if !temp_dir.exists() {
        std::fs::create_dir_all(temp_dir)?;
    }

    let status = container
        // Run hooks under fakeroot (fakefs).
        .command("/usr/bin/fakeroot")
        .arg("/usr/bin/drive_binary_package.sh")
        .arg("-r")
        .arg(root_dir)
        .arg("-d")
        .arg(format!("/{STAGE_DIR_NAME}"))
        .arg("-t")
        .arg(EBUILD_TEMP_DIR)
        .arg("-p")
        .arg(category_pf)
        .args(phases)
        .status()?;
    ensure!(status.success(), "Command failed: {:?}", status);

    Ok(())
}

/// Runs the pkg_setup and pkg_preinst hook. It returns `true` if it turned out that
/// we don't need to run hooks, including `pkg_postinst``.
/// An upper directory is saved to `preinst_dir`.
fn run_pkg_setup_and_preinst(
    settings: &ContainerSettings,
    preinst_dir: &Path,
    root_dir: &Path,
    category_pf: &str,
) -> Result<()> {
    let _span = info_span!("pkg_setup+pkg_preinst").entered();

    let mut container = settings.prepare()?;
    run_hooks_general(&mut container, root_dir, category_pf, &["setup", "preinst"])?;
    move_directory(&container.into_upper_dir(), preinst_dir)?;
    Ok(())
}

/// Removes the empty ancestor directories between the `child_dir` and the `root_dir`.
fn remove_empty_ancestors(root_dir: &Path, child_dir: &Path) -> Result<()> {
    for dir in child_dir.ancestors() {
        if dir == root_dir {
            break;
        }
        match std::fs::remove_dir(dir) {
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) if e.raw_os_error() == Some(libc::ENOTEMPTY) => break,
            other => other?,
        }
    }

    Ok(())
}

/// Post-processes a preinst layer and returns an initial upper directory to be used to run
/// pkg_postinst.
///
/// After running pkg_setup/pkg_preinst, we have to post-process the preinst layer for some reasons:
/// - Migrate modifications to $D (= "/.image") to the postinst layer so that they're applied
///   correctly to the package installation result.
/// - We have to update CONTENTS file in VDB if there were any modification to $D and
///  `sparse_vdb` is not set. If there are no changes we can simply use CONTENTS from the
///   installed contents layer.
fn mangle_preinst_layer(
    settings: &ContainerSettings,
    preinst_dir: &Path,
    root_dir: &Path,
    category_pf: &str,
    mutable_base_dir: &Path,
    sparse_vdb: bool,
) -> Result<SafeTempDir> {
    let _span = info_span!("mangle_preinst_layer").entered();

    // Prepare an upper directory for the postinst hook.
    let postinst_upper_dir = SafeTempDirBuilder::new()
        .base_dir(mutable_base_dir)
        .prefix("upper.")
        .build()?;

    // We have to specially handle $preinst_dir/.image that is created when pkg_setup or pkg_preinst
    // have made modifications to $D.
    let mut updated_contents: Option<Vec<u8>> = None;
    let preinst_image_dir = preinst_dir.join(STAGE_DIR_NAME);
    if preinst_image_dir.is_dir() {
        if !sparse_vdb {
            let _span = info_span!("recompute_contents").entered();

            // We have to recompute CONTENTS in the VDB directory.
            updated_contents = {
                let container = settings.prepare()?;
                let mut contents = Vec::new();
                generate_vdb_contents(&mut contents, &container.root_dir().join(STAGE_DIR_NAME))?;
                Some(contents)
            };
        }

        let _span = info_span!("move_contents").entered();

        // Use the image directory as the initial contents of the postinst upper directory.
        // This effectively applies preinst modifications to $D to the installed files.
        let postinst_root_dir = postinst_upper_dir.path().join(
            root_dir
                .strip_prefix("/")
                .expect("--root-dir must be absolute"),
        );
        std::fs::create_dir_all(&postinst_root_dir)?;
        move_directory(&preinst_image_dir, &postinst_root_dir)?;
    }

    // Migrate preinst modifications to the VDB directory to the postinst
    // upper directory to reduce preinst layer contents.
    let absolute_vdb_dir = get_vdb_dir(root_dir, category_pf);
    let relative_vdb_dir = absolute_vdb_dir
        .strip_prefix("/")
        .expect("get_vdb_dir must return an absolute path");
    let preinst_vdb_dir = preinst_dir.join(relative_vdb_dir);
    let postinst_vdb_dir = postinst_upper_dir.path().join(relative_vdb_dir);

    // We do this even though `sparse_vdb` is set because the postinst hook
    // might need this data. We instead strip the vdb after the postinst hook
    // has been run.
    if preinst_vdb_dir.exists() {
        std::fs::create_dir_all(&postinst_vdb_dir)?;
        move_directory(&preinst_vdb_dir, &postinst_vdb_dir)?;
    }

    // If we recomputed CONTENTS, it's time to apply it now that we've reflected VDB
    // modifications.
    if let Some(contents) = updated_contents {
        std::fs::write(postinst_vdb_dir.join("CONTENTS"), contents)?;
    }

    // Remove unnecessary directories from the preinst layer we created for VDB.
    remove_empty_ancestors(preinst_dir, &preinst_vdb_dir)?;

    Ok(postinst_upper_dir)
}

/// Runs the pkg_postinst hook. An upper directory is saved to `postinst_dir`.
fn run_pkg_postinst(
    settings: &ContainerSettings,
    postinst_dir: &Path,
    root_dir: &Path,
    category_pf: &str,
    upper_dir: SafeTempDir,
) -> Result<()> {
    let _span = info_span!("postinst").entered();

    let mut container = settings.prepare_with_upper_dir(upper_dir)?;
    run_hooks_general(&mut container, root_dir, category_pf, &["postinst"])?;
    move_directory(&container.into_upper_dir(), postinst_dir)?;
    Ok(())
}

/// Installs a package to the container in the sysroot at `root_dir`.
///
/// This function adds layers to `settings` so that the installed package is available in the
/// container.
fn install_package(
    settings: &mut ContainerSettings,
    spec: &InstallSpec,
    root_dir: &Path,
    mutable_base_dir: &Path,
    ensure_skip_hooks: bool,
    sparse_vdb: bool,
) -> Result<()> {
    let _span = info_span!(
        "install",
        package = ?spec.input_binary_package.file_name().unwrap(),
    )
    .entered();

    let binary_package = BinaryPackage::open(&spec.input_binary_package)
        .with_context(|| format!("Failed to open {}", spec.input_binary_package.display()))?;

    // We use the category_p as the category_pf because we want to elide the revision number.
    let category_pf = if sparse_vdb {
        binary_package.category_p()
    } else {
        binary_package.category_pf()
    };

    tracing::info!("Installing {}", category_pf);

    // Make sure output directories have right permissions.
    std::fs::set_permissions(&spec.output_preinst_dir, Permissions::from_mode(0o755))?;
    std::fs::set_permissions(&spec.output_postinst_dir, Permissions::from_mode(0o755))?;

    // Check if we can skip install hooks.
    if can_skip_install_hooks(
        &resolve_symlink_forest(&spec.input_staged_contents_dir)?,
        category_pf,
        root_dir,
    )? {
        let _span = info_span!("install_without_hooks", package = category_pf).entered();

        tracing::info!("Skipping install hooks safely");

        // Enough to just mount the installed contents layer.
        // TODO(b/299564235): Check file collisions.
        settings.push_layer(&resolve_symlink_forest(&spec.input_installed_contents_dir)?)?;

        return Ok(());
    }

    if ensure_skip_hooks {
        bail!("--ensure-skip-hooks was set but we need to run hooks");
    }

    // Bind-mount the binary package.
    bind_mount_binary_package(settings, &spec.input_binary_package, root_dir, category_pf)?;

    // Mount the staged contents at /.image.
    // This needs to be done before pkg_setup to provide VDB.
    settings.push_layer(&resolve_symlink_forest(&spec.input_staged_contents_dir)?)?;

    // Run pkg_setup and pkg_preinst.
    run_pkg_setup_and_preinst(settings, &spec.output_preinst_dir, root_dir, category_pf)?;

    // Add the preinst layer to the container so that pkg_postinst and packages installed later can
    // see modifications made by pkg_setup and pkg_preinst.
    settings.push_layer(&spec.output_preinst_dir)?;

    // Mangle the preinst layer and get the initial postinst upper dir.
    let postinst_upper_dir = mangle_preinst_layer(
        settings,
        &spec.output_preinst_dir,
        root_dir,
        category_pf,
        mutable_base_dir,
        sparse_vdb,
    )?;

    // Create an empty file to hide /.image. This file will be removed from the layer later.
    File::create(postinst_upper_dir.path().join(STAGE_DIR_NAME))?;

    // Mount the installed contents.
    // TODO(b/299564235): Check file collisions.
    settings.push_layer(&resolve_symlink_forest(&spec.input_installed_contents_dir)?)?;

    // Run pkg_postinst.
    run_pkg_postinst(
        settings,
        &spec.output_postinst_dir,
        root_dir,
        category_pf,
        postinst_upper_dir,
    )?;

    // When working with sparse vdbs, we drop any of the vdb modifications performed by the
    // hooks since we only have a few entries in a sparse vdb, and they are considered
    // immutable.
    if sparse_vdb {
        let vdb_dir = get_vdb_dir(
            root_dir
                .strip_prefix("/")
                .expect("--root-dir must return an absolute path"),
            category_pf,
        );
        let post_vdb_dir = spec.output_postinst_dir.join(vdb_dir);

        remove_dir_all(&post_vdb_dir).with_context(|| format!("rm -rf {post_vdb_dir:?}"))?;

        remove_empty_ancestors(
            &spec.output_postinst_dir,
            post_vdb_dir.parent().expect("vdb to have a parent"),
        )?;
    }

    // Add the postinst layer to the container so that packages installed later can see
    // modifications made by pkg_postinst.
    settings.push_layer(&spec.output_postinst_dir)?;

    Ok(())
}

fn postprocess_layers(spec: &InstallSpec) -> Result<()> {
    let _span = info_span!(
        "postprocess",
        package = ?spec.input_binary_package.file_name().unwrap(),
    )
    .entered();

    for output_dir in [&spec.output_preinst_dir, &spec.output_postinst_dir] {
        // Delete an empty .image file from layers which were used to hide image layers.
        let image_dir = output_dir.join(STAGE_DIR_NAME);
        if image_dir.try_exists()? {
            std::fs::remove_file(image_dir)?;
        }
        container::clean_layer(output_dir)?;
        DurableTree::convert(output_dir)?;
    }

    Ok(())
}

#[derive(Parser, Clone, Debug)]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Specifies the root directory to install packages at. It is typically
    /// "/" for host packages, or "/build/$BOARD" for target packages.
    #[arg(long)]
    root_dir: PathBuf,

    /// Specifies binary packages to install, in the order of installation. A value contains
    /// multiple file paths pointing to inputs and outputs related to a package.
    /// See [`InstallSpec`] for details.
    #[arg(long)]
    install: Vec<InstallSpec>,

    /// Abort if we have to run hooks. Used for testing.
    #[arg(long)]
    ensure_skip_hooks: bool,

    /// Specifies if we are working with sparse vdbs.
    ///
    /// This will drop the revision number from the vdb, and it will also skip
    /// generating a new `CONTENTS` entry.
    #[arg(long)]
    sparse_vdb: bool,
}

fn do_main() -> Result<()> {
    let args = Args::try_parse()?;

    // Mount a tmpfs and use it as the mutable base directory.
    //
    // We do this to workaround the issue where overlayfs blocks on unmounting to flush all dirty
    // writes to the file system of upper directories.
    // See: https://www.cloudfoundry.org/blog/an-overlayfs-journey-with-the-garden-team/
    //
    // This means that:
    // - File system modifications made by package hooks are recorded in memory (until they are
    //   finally moved to the output directory).
    // - Package hooks can not modify file ownership because fakefs fails to set xattrs.
    let tmpfs = TmpfsTempDir::new()?;

    let mut settings = ContainerSettings::new();
    settings.set_mutable_base_dir(tmpfs.path());
    settings.apply_common_args(&args.common)?;

    // Bind-mount the portageq cache script.
    let r = Runfiles::create()?;
    settings.push_bind_mount(BindMount {
        mount_path: PathBuf::from("/usr/local/bin/portageq"),
        source: runfiles::rlocation!(
            r,
            "cros/bazel/portage/bin/fast_install_packages/portageq_wrapper.py"
        ),
        rw: false,
    });

    for spec in &args.install {
        install_package(
            &mut settings,
            spec,
            &args.root_dir,
            tmpfs.path(),
            args.ensure_skip_hooks,
            args.sparse_vdb,
        )?;
    }

    for spec in &args.install {
        postprocess_layers(spec)?;
    }

    tracing::info!(
        "fast_install_packages: Installed {} packages",
        args.install.len()
    );

    Ok(())
}

fn main() -> ExitCode {
    enter_mount_namespace().expect("Failed to enter a mount namespace");
    cli_main(do_main, Default::default())
}
