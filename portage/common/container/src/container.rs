// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs::File,
    io::Read,
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    str::FromStr,
};

use anyhow::{bail, ensure, Result};
use durabletree::DurableTree;
use fileutil::{resolve_symlink_forest, SafeTempDir, SafeTempDirBuilder};
use run_in_container_lib::{BindMountConfig, RunInContainerConfig};
use strum_macros::EnumString;
use tracing::info_span;

use crate::{
    control::ControlChannel,
    mounts::{bind_mount, mount_overlayfs, remount_readonly, MountGuard},
};

const DEFAULT_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:\
    /sbin:/bin:/opt/bin:/mnt/host/source/chromite/bin:/mnt/host/depot_tools";

#[derive(Debug, Clone, Copy, PartialEq, EnumString, strum_macros::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum LoginMode {
    #[strum(serialize = "")]
    Never,
    Before,
    After,
    AfterFail,
}

#[derive(Clone, Debug)]
pub struct BindMount {
    pub mount_path: PathBuf,
    pub source: PathBuf,
    pub rw: bool,
}

impl FromStr for BindMount {
    type Err = anyhow::Error;

    fn from_str(spec: &str) -> Result<Self> {
        let v: Vec<_> = spec.split('=').collect();
        ensure!(v.len() == 2, "Invalid bind-mount spec: {:?}", spec);
        Ok(Self {
            mount_path: v[0].into(),
            source: v[1].into(),
            rw: false,
        })
    }
}

impl BindMount {
    pub fn into_config(self) -> BindMountConfig {
        BindMountConfig {
            mount_path: self.mount_path,
            source: self.source,
            rw: self.rw,
        }
    }
}

/// Implements the parser of command line options common to CLIs that make use
/// of containers.
///
/// Include this struct in your struct that derives [`clap::Parser`], and
/// annotate the field with `#[command(flatten)]` to inherit the options
/// declared in this struct.
///
/// # Example
///
/// ```
/// #[derive(clap::Parser)]
/// struct Cli {
///     #[command(flatten)]
///     common: CommonArgs,
///
///     another_arg: bool,
/// }
/// ```
#[derive(Clone, Debug, clap::Args)]
pub struct CommonArgs {
    #[arg(
        long,
        help = "Adds a file system layer to be mounted in the container."
    )]
    pub layer: Vec<PathBuf>,

    #[arg(
        long,
        help = "Internal flag used to differentiate between a normal \
            invocation and a user invocation. i.e., _debug targets",
        hide = true, // We only want the _debug targets setting this flag.
    )]
    pub interactive: bool,

    #[arg(
        long,
        help = "Logs in to the SDK before installing deps, before building, \
            after building, or after failing to build respectively.",
        default_value_if("interactive", "true", Some("after")),
        default_value_t = LoginMode::Never,
    )]
    pub login: LoginMode,

    #[arg(
        long,
        help = "Keeps the host file system at /host. Use for debuggin only."
    )]
    pub keep_host_mount: bool,
}

#[derive(Clone, Debug)]
enum LayerType {
    Archive,
    Dir,
    DurableTree,
}

impl LayerType {
    fn detect(path: &Path) -> Result<Self> {
        let file_name = path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        if DurableTree::try_exists(path)? {
            Ok(LayerType::DurableTree)
        } else if std::fs::metadata(path)?.is_dir() {
            Ok(LayerType::Dir)
        } else if file_name.ends_with(".tar.zst")
            || file_name.ends_with(".tar.gz")
            || file_name.ends_with(".tar")
        {
            Ok(LayerType::Archive)
        } else {
            bail!("unsupported file type: {}", path.display());
        }
    }
}

/// Holds settings to construct containers, such as file system layers and bind
/// mounts.
///
/// This is a builder-like object. Call mutable methods to update settings and
/// call [`ContainerSettings::prepare`] to create a [`PreparedContainer`]. The
/// same [`ContainerSettings`] can be reused multiple times to create different
/// [`PreparedContainer`] objects.
pub struct ContainerSettings {
    mutable_base_dir: PathBuf,
    allow_network_access: bool,
    login_mode: LoginMode,
    keep_host_mount: bool,
    lower_dirs: Vec<PathBuf>,
    archive_dirs: Vec<SafeTempDir>,
    durable_trees: Vec<DurableTree>,
    reusable_archive_dir: Option<PathBuf>,
    bind_mounts: Vec<BindMount>,
}

impl ContainerSettings {
    /// Creates an empty [`ContainerSettings`].
    pub fn new() -> Self {
        Self {
            mutable_base_dir: std::env::temp_dir(),
            allow_network_access: false,
            login_mode: LoginMode::Never,
            keep_host_mount: false,
            lower_dirs: Vec::new(),
            archive_dirs: Vec::new(),
            durable_trees: Vec::new(),
            reusable_archive_dir: None,
            bind_mounts: Vec::new(),
        }
    }

    /// Specifies the *mutable base directory* where an upper directory and a
    /// scratch directory are created for containers.
    ///
    /// By default, the mutable base directory is `$TMPDIR`. You can set it to
    /// another directory if you want to move an upper directory to another
    /// location without copying them across file system boundaries.
    pub fn set_mutable_base_dir(&mut self, mutable_base_dir: &Path) {
        self.mutable_base_dir = mutable_base_dir.to_owned();
    }

    /// Sets whether to allow network access to processes in the container.
    /// This option should be enabled only when it's absolutely needed since it
    /// reduces hermeticity of the container.
    pub fn set_allow_network_access(&mut self, allow_network_access: bool) {
        self.allow_network_access = allow_network_access;
    }

    /// Sets the login mode for containers.
    ///
    /// If it is set to a value other than [`LoginMode::Never`], the container
    /// will start an interactive shell on the specified occasion for debugging.
    pub fn set_login_mode(&mut self, login_mode: LoginMode) {
        self.login_mode = login_mode;
    }

    /// Specifies whether to leave the host file system at `/host` in
    /// containers.
    ///
    /// The default is false. Do not enable this option unless you need to debug
    /// container issues because exposing host file system to the container
    /// harms its hermeticity.
    pub fn set_keep_host_mount(&mut self, keep_host_mount: bool) {
        self.keep_host_mount = keep_host_mount;
    }

    /// Pushes a new layer to the container settings.
    ///
    /// This function prepares a layer by extracting archives and/or mounting
    /// underlying file filesystems of the layer. Therefore, the current process
    /// must have privilege to mount file systems, usually by calling
    /// [`enter_mount_namespace`](crate::namespace::enter_mount_namespace).
    pub fn push_layer(&mut self, path: &Path) -> Result<()> {
        let layer_type = LayerType::detect(path)?;

        let _span = info_span!("push_layer", ?layer_type, ?path).entered();

        match layer_type {
            LayerType::Archive => {
                let archive_dir = self.request_archive_dir()?;
                Self::extract_archive(path, &archive_dir)?;
                Ok(())
            }
            LayerType::Dir => {
                self.lower_dirs.push(path.to_owned());
                self.reusable_archive_dir = None;
                Ok(())
            }
            LayerType::DurableTree => {
                let durable_tree = DurableTree::expand(path)?;
                self.lower_dirs
                    .extend(durable_tree.layers().into_iter().map(ToOwned::to_owned));
                self.durable_trees.push(durable_tree);
                self.reusable_archive_dir = None;
                Ok(())
            }
        }
    }

    /// Pushes a new bind mount to the container settings.
    pub fn push_bind_mount(&mut self, bind_mount: BindMount) {
        self.bind_mounts.push(bind_mount);
    }

    /// Applies container settings represented in [`CommonArgs`].
    pub fn apply_common_args(&mut self, args: &CommonArgs) -> Result<()> {
        self.set_keep_host_mount(args.keep_host_mount);
        self.set_login_mode(args.login);

        for path in args.layer.iter() {
            self.push_layer(&resolve_symlink_forest(path)?)?;
        }
        Ok(())
    }

    /// Prepares to start a container by creating necessary directories such as
    /// an upper directory.
    ///
    /// The returned [`PreparedContainer`] instance can be used to actually
    /// start a process in the container. Since [`PreparedContainer`] borrows
    /// [`ContainerSettings`], the borrow checker prevents you from updating the
    /// container settings until all [`PreparedContainer`] instances derived
    /// from it are destructed.
    pub fn prepare(&self) -> Result<PreparedContainer> {
        let upper_dir = SafeTempDirBuilder::new()
            .base_dir(&self.mutable_base_dir)
            .prefix("upper.")
            .build()?;
        PreparedContainer::new(self, upper_dir)
    }

    /// Prepares to start a container with a given upper directory.
    ///
    /// This is similar to [`prepare`], but callers can provide an upper
    /// directory that may contain initial contents. Note that, the upper
    /// directory must be under the same file system as the mutable base
    /// directory due to requirements of overlayfs.
    ///
    /// This function takes ownership of the given upper directory. On success,
    /// the returned [`PreparedContainer`] has the ownership of the directory
    /// and deletes it on drop. On failure, it deletes the directory
    /// immediately.
    pub fn prepare_with_upper_dir(&self, upper_dir: SafeTempDir) -> Result<PreparedContainer> {
        PreparedContainer::new(self, upper_dir)
    }

    fn request_archive_dir(&mut self) -> Result<PathBuf> {
        if let Some(reusable_archive_dir) = &self.reusable_archive_dir {
            Ok(reusable_archive_dir.clone())
        } else {
            let new_archive_dir = SafeTempDir::new()?;
            let path = new_archive_dir.path().to_owned();
            self.archive_dirs.push(new_archive_dir);
            self.reusable_archive_dir = Some(path.clone());
            self.lower_dirs.push(path.clone());
            Ok(path)
        }
    }

    fn extract_archive(archive_path: &Path, extract_dir: &Path) -> Result<()> {
        let f = File::open(archive_path)?;
        let decompressed: Box<dyn Read> = match archive_path.extension() {
            Some(s) if s == OsStr::new("zst") => Box::new(zstd::stream::read::Decoder::new(f)?),
            Some(s) if s == OsStr::new("gz") => Box::new(flate2::read::GzDecoder::new(f)),
            _ => Box::new(f),
        };
        tar::Archive::new(decompressed).unpack(extract_dir)?;
        Ok(())
    }
}

impl Default for ContainerSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a container prepared for execution.
///
/// The most interesting method is [`PreparedContainer::command`] that allows
/// you to start a command in the container. You can call it multiple times as
/// you like.
///
/// Also, a prepared container comes with an overlayfs mount whose path can be
/// obtained by [`PreparedContainer::root_dir`]. You can inspect or modify
/// the contents of the overlay file system freely.
///
/// After you are done with the container, you have two choices:
/// 1. Just drop `PreparedContainer`. Then changes to the container are lost.
/// 2. Call [`PreparedContainer::into_upper_dir`] to convert it to its upper
///    directory path. This is useful if you want to persist the change to the
///    container.
pub struct PreparedContainer<'settings> {
    settings: &'settings ContainerSettings,

    // Note: The order of fields matters here!
    // `_overlayfs_guard` must come before `upper_dir` and `scratch_dir` so that
    // the overlayfs is unmounted before removing its backing directories.
    _overlayfs_guard: MountGuard,
    root_dir: SafeTempDir,
    _stage_dir: SafeTempDir,
    _scratch_dir: SafeTempDir,
    upper_dir: SafeTempDir,

    base_envs: BTreeMap<OsString, OsString>,
}

impl<'settings> PreparedContainer<'settings> {
    fn new(settings: &'settings ContainerSettings, upper_dir: SafeTempDir) -> Result<Self> {
        let mut base_envs: BTreeMap<OsString, OsString> = BTreeMap::from_iter([
            ("PATH".into(), DEFAULT_PATH.into()),
            // Always enable Rust backtrace.
            ("RUST_BACKTRACE".into(), "1".into()),
        ]);
        if settings.login_mode != LoginMode::Never {
            base_envs.insert("_LOGIN_MODE".into(), settings.login_mode.to_string().into());

            // Ensure we forward the TERM variable so bash behaves correctly.
            if let Some(term) = std::env::var_os("TERM") {
                base_envs.insert("_TERM".into(), term);
            }
        }

        // A stage directory is the most significant lower directory where we
        // inject necessary files/directories.
        let stage_dir = SafeTempDirBuilder::new()
            .base_dir(&settings.mutable_base_dir)
            .prefix("stage.")
            .build()?;

        // Create mount points for essential top-level directories.
        for d in ["dev", "proc", "sys", "tmp", "host"] {
            std::fs::create_dir(stage_dir.path().join(d))?;
        }

        // Copy `setup.sh` to `/.setup.sh`.
        let runfiles = runfiles::Runfiles::create()?;
        let setup_sh_path = stage_dir.path().join(".setup.sh");
        std::fs::copy(
            runfiles.rlocation("cros/bazel/portage/common/container/setup.sh"),
            &setup_sh_path,
        )?;
        std::fs::set_permissions(&setup_sh_path, PermissionsExt::from_mode(0o755))?;

        // Create mount points for bind-mounts.
        for spec in settings.bind_mounts.iter() {
            let target = stage_dir.path().join(spec.mount_path.strip_prefix("/")?);
            let metadata = std::fs::metadata(&spec.source)?;
            if metadata.is_dir() {
                std::fs::create_dir_all(&target)?;
            } else {
                std::fs::create_dir_all(target.parent().unwrap())?;
                File::create(&target)?;
            }
        }

        let scratch_dir = SafeTempDirBuilder::new()
            .base_dir(&settings.mutable_base_dir)
            .prefix("scratch.")
            .build()?;
        let root_dir = SafeTempDirBuilder::new()
            .base_dir(&settings.mutable_base_dir)
            .prefix("root")
            .build()?;

        let lower_dirs: Vec<&Path> = settings
            .lower_dirs
            .iter()
            .map(|p| p.as_path())
            .chain([stage_dir.path()])
            .collect();

        // Mount the overlayfs.
        let overlayfs_guard = mount_overlayfs(
            root_dir.path(),
            &lower_dirs,
            upper_dir.path(),
            scratch_dir.path(),
        )?;

        // Perform bind-mounts.
        for spec in settings.bind_mounts.iter() {
            let target = root_dir.path().join(spec.mount_path.strip_prefix("/")?);

            // Unfortunately, the MS_RDONLY is ignored for bind-mounts.
            // Thus, we mount a bind-mount, then remount it as readonly.
            bind_mount(&spec.source, &target)?.leak();
            if !spec.rw {
                remount_readonly(&target)?;
            }
        }

        // Bind-mount special file systems. Note that we don't need to umount
        // them on errors because they're under `root_dir` and recursively
        // unmounted by `overlayfs_guard`.
        bind_mount(Path::new("/dev"), &root_dir.path().join("dev"))?.leak();
        bind_mount(Path::new("/sys"), &root_dir.path().join("sys"))?.leak();

        // Note that we don't mount /proc here but it is mounted by
        // run_in_container instead. It is because we need privileges on the
        // current PID namespace to mount one, but entering a new PID namespace
        // in unit tests is not straightforward as it requires forking.

        Ok(Self {
            settings,
            _overlayfs_guard: overlayfs_guard,
            root_dir,
            _stage_dir: stage_dir,
            _scratch_dir: scratch_dir,
            upper_dir,
            base_envs,
        })
    }

    /// Returns the directory that subprocesses will see as the filesystem root.
    pub fn root_dir(&self) -> &Path {
        self.root_dir.path()
    }

    /// Creates a [`ContainerCommand`] that can be used to run a command within
    /// the container.
    ///
    /// Note: This method takes &mut self to prevent multiple commands from
    /// running in parallel. It is because it can cause the upper directory
    /// to be shared across multiple overlayfs and its behavior is undefined.
    /// To support running multiple commands in parallel, it is likely that
    /// `PreparedContainer` needs to mount overlayfs.
    pub fn command(&mut self, name: impl AsRef<OsStr>) -> ContainerCommand {
        ContainerCommand::new(self, name.as_ref(), &self.base_envs)
    }

    /// Destructs the prepared container and returns the path to its upper
    /// directory.
    ///
    /// It is your responsibility to remove the returned upper directory once
    /// you are done with it.
    #[must_use]
    pub fn into_upper_dir(self) -> PathBuf {
        self.upper_dir.into_path()
    }
}

/// Runs a command in a container.
///
/// The interface follows [`std::process::Command`].
pub struct ContainerCommand<'container> {
    container: &'container PreparedContainer<'container>,
    args: Vec<OsString>,
    envs: BTreeMap<OsString, OsString>,
    current_dir: PathBuf,
}

impl<'container> ContainerCommand<'container> {
    fn new(
        container: &'container PreparedContainer<'container>,
        name: &OsStr,
        base_envs: &BTreeMap<OsString, OsString>,
    ) -> Self {
        Self {
            container,
            args: vec![name.to_owned()],
            envs: base_envs.clone(),
            current_dir: PathBuf::from("/"),
        }
    }

    /// Sets the current directory of the process.
    pub fn current_dir<P>(&mut self, path: P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self.current_dir = path.as_ref().to_owned();
        self
    }

    /// Adds an argument to the process.
    pub fn arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<OsStr>,
    {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    /// Adds arguments to the process.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            self.arg(arg);
        }
        self
    }

    /// Adds an environment variable to the process.
    pub fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.envs
            .insert(key.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    /// Adds environment variables to the process.
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (key, value) in vars {
            self.env(key, value);
        }
        self
    }

    /// Runs a process in the container and returns its exit status.
    pub fn status(&mut self) -> Result<ExitStatus> {
        let _span = info_span!("status").entered();

        let mut real_args = vec!["/.setup.sh".into()];
        real_args.extend(self.args.clone());

        // This is the config passed to run_in_container.
        let config = RunInContainerConfig {
            args: real_args,
            root_dir: self.container.root_dir.path().to_path_buf(),
            envs: self.envs.clone(),
            chdir: self.current_dir.clone(),
            allow_network_access: self.container.settings.allow_network_access,
            keep_host_mount: self.container.settings.keep_host_mount,
        };

        // Save run_in_container.json.
        let config_dir = SafeTempDirBuilder::new()
            .base_dir(&self.container.settings.mutable_base_dir)
            .prefix("config")
            .build()?;
        let config_path = config_dir.path().join("run_in_container.json");
        config.serialize_to(&config_path)?;

        // Start a control channel for interactive shells if needed.
        let _control = if self.container.settings.login_mode == LoginMode::Never {
            None
        } else {
            Some(ControlChannel::new(
                self.container.root_dir.path().join(".control"),
            )?)
        };

        // Now it's time to start a container!
        let runfiles = runfiles::Runfiles::create()?;
        let run_in_container_path =
            runfiles.rlocation("cros/bazel/portage/bin/run_in_container/run_in_container");
        let status = processes::run(
            Command::new(run_in_container_path)
                .arg("--config")
                .arg(&config_path),
        )?;

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use super::*;

    /// Bind-mounts a statically-linked bash to the container.
    /// This is required for the container to work properly.
    fn bind_mount_bash(settings: &mut ContainerSettings) -> Result<()> {
        let runfiles = runfiles::Runfiles::create()?;
        settings.push_bind_mount(BindMount {
            mount_path: PathBuf::from("/bin/bash"),
            source: runfiles.rlocation("files/bash-static"),
            rw: false,
        });
        Ok(())
    }

    /// Asserts the content of a file in the container.
    fn assert_content(container: &mut PreparedContainer, path: &Path, content: &str) -> Result<()> {
        let script = format!(
            r#"exec < '{}'; read -r msg; [[ "${{msg}}" == '{}' ]]"#,
            path.display(),
            content
        );

        let status = container.command("bash").arg("-c").arg(script).status()?;
        assert!(status.success());
        Ok(())
    }

    #[test]
    fn test_success() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        let mut container = settings.prepare()?;

        let status = container.command("bash").args(["-c", ":"]).status()?;
        assert!(status.success());
        Ok(())
    }

    #[test]
    fn test_failure() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        let mut container = settings.prepare()?;

        let status = container.command("bash").args(["-c", "exit 28"]).status()?;
        assert_eq!(status.code(), Some(28));
        Ok(())
    }

    #[test]
    fn test_current_dir() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        let mut container = settings.prepare()?;

        // By default, the current directory is /.
        let status = container
            .command("bash")
            .args(["-c", "[[ \"${PWD}\" == / ]]"])
            .status()?;
        assert!(status.success());

        // Call `ContainerCommand::current_dir` to set a current directory.
        let status = container
            .command("bash")
            .args(["-c", "[[ \"${PWD}\" == /bin ]]"])
            .current_dir("/bin")
            .status()?;
        assert!(status.success());

        Ok(())
    }

    #[test]
    fn test_env() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        let mut container = settings.prepare()?;

        let status = container
            .command("bash")
            .args(["-c", "[[ \"${HELLO}\" == world ]]"])
            .env("HELLO", "world")
            .status()?;
        assert!(status.success());

        Ok(())
    }

    #[test]
    fn test_keep_host_mount() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        // By default, /host is empty.
        {
            let mut container = settings.prepare()?;
            let status = container
                .command("bash")
                .args(["-c", "[[ -d /host/proc ]]"])
                .status()?;
            assert!(!status.success());
        }

        // By enabling keep_host_mount, /host reveals the host file system.
        settings.set_keep_host_mount(true);
        {
            let mut container = settings.prepare()?;
            let status = container
                .command("bash")
                .args(["-c", "[[ -d /host/proc ]]"])
                .status()?;
            assert!(status.success());
        }

        Ok(())
    }

    #[test]
    fn test_bind_mount() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        let temp_dir = SafeTempDir::new()?;
        File::create(temp_dir.path().join("ok"))?;
        nix::unistd::mkfifo(&temp_dir.path().join("fifo"), nix::sys::stat::Mode::S_IRWXU)?;

        // Create bind mounts of a directory and a regular file.
        // In both cases, mount points should be created automatically.
        settings.push_bind_mount(BindMount {
            mount_path: PathBuf::from("/bind1"),
            source: temp_dir.path().to_owned(),
            rw: false,
        });
        settings.push_bind_mount(BindMount {
            mount_path: PathBuf::from("/bind2/ok"),
            source: temp_dir.path().join("ok"),
            rw: false,
        });
        settings.push_bind_mount(BindMount {
            mount_path: PathBuf::from("/bind3/fifo"),
            source: temp_dir.path().join("fifo"),
            rw: false,
        });

        let mut container = settings.prepare()?;
        let status = container
            .command("bash")
            .args([
                "-c",
                "[[ -f /bind1/ok ]] && [[ -f /bind2/ok ]] && [[ -p /bind3/fifo ]]",
            ])
            .status()?;
        assert!(status.success());

        Ok(())
    }

    #[test]
    fn test_bind_mount_read_write() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        let temp_dir = SafeTempDir::new()?;
        File::create(temp_dir.path().join("file"))?;

        // Create two bind mounts where one is read-only while the other is
        // read-write.
        settings.push_bind_mount(BindMount {
            mount_path: PathBuf::from("/bind-ro"),
            source: temp_dir.path().to_owned(),
            rw: false,
        });
        settings.push_bind_mount(BindMount {
            mount_path: PathBuf::from("/bind-rw"),
            source: temp_dir.path().to_owned(),
            rw: true,
        });

        let mut container = settings.prepare()?;

        // Writing to /bind-ro fails.
        let status = container
            .command("bash")
            .args(["-c", ": > /bind-ro/file"])
            .status()?;
        assert!(!status.success());

        // Writing to /bind-rw succeeds.
        let status = container
            .command("bash")
            .args(["-c", ": > /bind-rw/file"])
            .status()?;
        assert!(status.success());

        Ok(())
    }

    #[test]
    fn test_layers() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        // Create a durable tree at run time.
        let durable_tree_dir = SafeTempDir::new()?;
        let durable_tree_dir = durable_tree_dir.path();
        std::fs::write(
            durable_tree_dir.join("hello.txt"),
            "This file is from the durable tree layer.",
        )?;
        DurableTree::convert(durable_tree_dir)?;
        DurableTree::cool_down_for_testing(durable_tree_dir)?;

        let hello_path = Path::new("/hello.txt");
        let runfiles = runfiles::Runfiles::create()?;

        // Push the directory layer.
        settings.push_layer(&resolve_symlink_forest(
            &runfiles.rlocation("cros/bazel/portage/common/container/testdata/layer-dir"),
        )?)?;
        assert_content(
            &mut settings.prepare()?,
            hello_path,
            "This file is from the directory layer.",
        )?;

        // Push the archive layer.
        settings.push_layer(
            &runfiles
                .rlocation("cros/bazel/portage/common/container/testdata/layer-archive.tar.zst"),
        )?;
        assert_content(
            &mut settings.prepare()?,
            hello_path,
            "This file is from the archive layer.",
        )?;

        // Push the durable tree layer.
        settings.push_layer(durable_tree_dir)?;
        assert_content(
            &mut settings.prepare()?,
            hello_path,
            "This file is from the durable tree layer.",
        )?;

        Ok(())
    }

    #[test]
    fn test_upper_to_lower() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        // Set the mutable base directory explicitly to clear upper directories
        // at once.
        let mutable_base_dir = SafeTempDir::new()?;
        let mutable_base_dir = mutable_base_dir.path();
        settings.set_mutable_base_dir(mutable_base_dir);

        // Run a container which creates a file.
        let mut container = settings.prepare()?;
        assert!(container
            .command("bash")
            .args(["-c", "echo ok > /file"])
            .status()?
            .success());

        // Extract the upper directory and push it as a new lower directory.
        let upper_dir = container.into_upper_dir();
        settings.push_layer(&upper_dir)?;

        // Run a container, and it should see the new file.
        let mut container = settings.prepare()?;
        assert_content(&mut container, Path::new("/file"), "ok")?;

        Ok(())
    }

    #[test]
    fn test_archive_layers_merge() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        let runfiles = runfiles::Runfiles::create()?;

        // Push an archive layer, a directory layer, and another archive layer.
        // ContainerSettings should not merge non-adjacent archive layers.
        settings.push_layer(
            &runfiles
                .rlocation("cros/bazel/portage/common/container/testdata/layer-archive.tar.zst"),
        )?;
        settings.push_layer(&resolve_symlink_forest(
            &runfiles.rlocation("cros/bazel/portage/common/container/testdata/layer-dir"),
        )?)?;
        settings.push_layer(
            &runfiles
                .rlocation("cros/bazel/portage/common/container/testdata/layer-archive.tar.zst"),
        )?;

        assert_content(
            &mut settings.prepare()?,
            Path::new("/hello.txt"),
            "This file is from the archive layer.",
        )?;

        Ok(())
    }

    #[test]
    fn test_initial_upper_dir() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        let upper_dir = SafeTempDir::new()?;
        std::fs::write(upper_dir.path().join("hello.txt"), "Hello, world!")?;

        assert_content(
            &mut settings.prepare_with_upper_dir(upper_dir)?,
            Path::new("/hello.txt"),
            "Hello, world!",
        )?;

        Ok(())
    }

    #[test]
    fn test_common_args() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        settings.apply_common_args(&CommonArgs {
            layer: vec![PathBuf::from(
                "bazel/portage/common/container/testdata/layer-dir",
            )],
            interactive: false,
            login: LoginMode::Never,
            keep_host_mount: false,
        })?;

        assert_content(
            &mut settings.prepare()?,
            Path::new("/hello.txt"),
            "This file is from the directory layer.",
        )?;

        Ok(())
    }

    #[test]
    fn test_resolve_symlink_forests() -> Result<()> {
        let mut settings = ContainerSettings::new();
        bind_mount_bash(&mut settings)?;

        // Simulate a symlink forest.
        let actual_dir = SafeTempDir::new()?;
        let actual_dir = actual_dir.path();
        std::fs::write(actual_dir.join("hello.txt"), "world")?;
        let forest_dir = SafeTempDir::new()?;
        let forest_dir = forest_dir.path();
        symlink(actual_dir.join("hello.txt"), forest_dir.join("hello.txt"))?;

        settings.apply_common_args(&CommonArgs {
            layer: vec![forest_dir.to_owned()],
            interactive: false,
            login: LoginMode::Never,
            keep_host_mount: false,
        })?;

        assert_content(&mut settings.prepare()?, Path::new("/hello.txt"), "world")?;

        Ok(())
    }
}
