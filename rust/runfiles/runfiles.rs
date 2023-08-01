//! Runfiles lookup library for Bazel-built Rust binaries and tests.
//!
//! USAGE:
//!
//! 1.  Depend on this runfiles library from your build rule:
//!     ```python
//!       rust_binary(
//!           name = "my_binary",
//!           ...
//!           data = ["//path/to/my/data.txt"],
//!           deps = ["@rules_rust//tools/runfiles"],
//!       )
//!     ```
//!
//! 2.  Import the runfiles library.
//!     ```ignore
//!     extern crate runfiles;
//!
//!     use runfiles::Runfiles;
//!     ```
//!
//! 3.  Create a Runfiles object and use rlocation to look up runfile paths:
//!     ```ignore -- This doesn't work under rust_doc_test because argv[0] is not what we expect.
//!
//!     use runfiles::Runfiles;
//!
//!     let r = Runfiles::create().unwrap();
//!     let path = r.rlocation("my_workspace/path/to/my/data.txt");
//!
//!     let f = File::open(path).unwrap();
//!     // ...
//!     ```

use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

const RUNFILES_DIR_ENV_VAR: &str = "RUNFILES_DIR";
const MANIFEST_FILE_ENV_VAR: &str = "RUNFILES_MANIFEST_FILE";
const TEST_SRCDIR_ENV_VAR: &str = "TEST_SRCDIR";

#[derive(Debug)]
enum Mode {
    DirectoryBased(PathBuf),
    ManifestBased(HashMap<PathBuf, PathBuf>),
}

type RepoMapping = HashMap<(String, String), String>;

#[derive(Debug)]
pub struct Runfiles {
    mode: Mode,
    repo_mapping: Option<RepoMapping>,
}

impl Runfiles {
    /// Creates a manifest based Runfiles object when
    /// RUNFILES_MANIFEST_ONLY environment variable is present,
    /// or a directory based Runfiles object otherwise.
    pub fn create() -> io::Result<Self> {
        if let Some(runfiles_dir) = read_env_path(RUNFILES_DIR_ENV_VAR) {
            Self::create_directory_based(runfiles_dir)
        } else if let Some(manifest) = read_env_path(MANIFEST_FILE_ENV_VAR) {
            Self::create_manifest_based(manifest)
        } else {
            Self::create_from_exec_path()
        }

    }

    fn create_directory_based(runfiles_dir: PathBuf) -> io::Result<Self> {
        if runfiles_dir.is_dir() {
            let repo_mapping = match std::fs::read_to_string(runfiles_dir.join("_repo_mapping")) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e),
                Ok(contents) => Some(parse_repo_mapping(contents)?),
            };
            Ok(Runfiles {
                repo_mapping,
                mode: Mode::DirectoryBased(runfiles_dir),
            })
        } else {
            Err(make_io_error("Specified runfiles directory doesn't exist."))
        }
    }

    fn create_manifest_based(manifest_path: PathBuf) -> io::Result<Self> {
        let manifest_content = std::fs::read_to_string(manifest_path)?;
        let path_mapping = manifest_content
            .lines()
            .map(|line| {
                let pair = line
                    .split_once(' ')
                    .expect("manifest file contained unexpected content");
                (pair.0.into(), pair.1.into())
            })
            .collect::<HashMap<_, _>>();
        Ok(Runfiles {
            repo_mapping: match path_mapping.get(&PathBuf::from("_repo_mapping")) {
                Some(path) => Some(parse_repo_mapping(std::fs::read_to_string(path)?)?),
                None => None,
            },
            mode: Mode::ManifestBased(path_mapping),
        })
    }

    /// Returns the runtime path of a runfile.
    ///
    /// Runfiles are data-dependencies of Bazel-built binaries and tests.
    /// The returned path may not be valid. The caller should check the path's
    /// validity and that the path exists.
    pub fn rlocation(&self, path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();
        let path = self
            .repo_mapping
            .as_ref()
            .and_then(|mapping| {
                let mut components = path.components();
                if let Some(std::path::Component::Normal(target_local)) = components.next() {
                    if let Some(target_local) = target_local.to_str() {
                        let current_canonical = self.current_repository();
                        // With bzlmod, the current repository for the main directory is always "_main",
                        // but in the repo mapping it's listed as the empty string.
                        let current_canonical_or_blank = match current_canonical {
                            "_main" => "",
                            other => other,
                        };
                        let target_canonical = mapping
                            .get(&(current_canonical_or_blank.into(), target_local.into()))
                            .unwrap_or_else(|| {
                                panic!(
                                    "Repo {} is not visible from {} in the repo mapping",
                                    target_local, current_canonical
                                );
                            });
                        return Some(Path::new(target_canonical).join(components.as_path()));
                    }
                }
                None
            })
            .unwrap_or_else(|| path.to_path_buf());
        match &self.mode {
            Mode::DirectoryBased(runfiles_dir) => runfiles_dir.join(path),
            Mode::ManifestBased(path_mapping) => path_mapping
                .get(&path)
                .unwrap_or_else(|| {
                    panic!("Path {} not found among runfiles.", path.to_string_lossy())
                })
                .clone(),
        }
    }

    /// Returns the canonical name of the caller's Bazel repository.
    pub fn current_repository(&self) -> &str {
        // This value must match the value of `_RULES_RUST_RUNFILES_WORKSPACE_NAME`
        // which can be found in `@rules_rust//tools/runfiles/private:workspace_name.bzl`
        env!("RULES_RUST_RUNFILES_WORKSPACE_NAME")
    }

    /// Returns the .runfiles directory for the currently executing binary.
    pub fn create_from_exec_path() -> io::Result<Self> {
        // Consume the first argument (argv[0])
        let exec_path = std::env::args().next().expect("arg 0 was not set");

        let mut binary_path = PathBuf::from(&exec_path);
        loop {
            // Check for our neighboring $binary.runfiles directory.
            let mut runfiles_name = binary_path.file_name().unwrap().to_owned();
            runfiles_name.push(".runfiles");
            let mut manifest_name = binary_path.file_name().unwrap().to_owned();
            manifest_name.push(".runfiles_manifest");

            let runfiles_path = binary_path.with_file_name(&runfiles_name);
            if runfiles_path.is_dir() {
                return Self::create_directory_based(runfiles_path)
            }
            let manifest_path = binary_path.with_file_name(&manifest_name);
            if manifest_path.is_file() {
                return Self::create_manifest_based(manifest_path)
            }

            // Check if we're already under a *.runfiles directory.
            {
                // TODO: 1.28 adds Path::ancestors() which is a little simpler.
                let mut next = binary_path.parent();
                while let Some(ancestor) = next {
                    if ancestor
                        .file_name()
                        .map_or(false, |f| f.to_string_lossy().ends_with(".runfiles"))
                    {
                        return Self::create_directory_based(ancestor.to_path_buf());
                    }
                    next = ancestor.parent();
                }
            }

            if !fs::symlink_metadata(&binary_path)?.file_type().is_symlink() {
                break;
            }
            // Follow symlinks and keep looking.
            let link_target = binary_path.read_link()?;
            binary_path = if link_target.is_absolute() {
                link_target
            } else {
                let link_dir = binary_path.parent().unwrap();
                env::current_dir()?.join(link_dir).join(link_target)
            }
        }

        Err(make_io_error("failed to find .runfiles directory"))
    }

}

fn parse_repo_mapping(contents: String) -> io::Result<RepoMapping> {
    contents
        .lines()
        .map(|line| match line.splitn(3, ',').collect::<Vec<_>>()[..] {
            [current_canonical, target_local, target_canonical] => Ok((
                (current_canonical.to_string(), target_local.to_string()),
                target_canonical.to_string(),
            )),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Repo mapping file contained unexpected content",
            )),
        })
        .collect::<io::Result<RepoMapping>>()
}

fn read_env_path(key: &str) -> Option<PathBuf> {
    std::env::var(key).ok().map(PathBuf::from)
}

fn make_io_error(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

#[cfg(test)]
mod test {
    use super::*;

    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_can_read_data_from_runfiles() {
        // We want to run multiple test cases with different environment variables set. Since
        // environment variables are global state, we need to ensure the two test cases do not run
        // concurrently. Rust runs tests in parallel and does not provide an easy way to synchronise
        // them, so we run all test cases in the same #[test] function.

        let test_srcdir =
            env::var_os(TEST_SRCDIR_ENV_VAR).expect("bazel did not provide TEST_SRCDIR");
        let runfiles_dir =
            env::var_os(RUNFILES_DIR_ENV_VAR).expect("bazel did not provide RUNFILES_DIR");

        // Test case 1: Only $RUNFILES_DIR is set.
        {
            env::remove_var(TEST_SRCDIR_ENV_VAR);
            let r = Runfiles::create().unwrap();

            let mut f =
                File::open(r.rlocation("rules_rust/tools/runfiles/data/sample.txt")).unwrap();

            let mut buffer = String::new();
            f.read_to_string(&mut buffer).unwrap();

            assert_eq!("Example Text!", buffer);
            env::set_var(TEST_SRCDIR_ENV_VAR, &test_srcdir)
        }
        // Test case 2: Only $TEST_SRCDIR is set.
        {
            env::remove_var(RUNFILES_DIR_ENV_VAR);
            let r = Runfiles::create().unwrap();

            let mut f =
                File::open(r.rlocation("rules_rust/tools/runfiles/data/sample.txt")).unwrap();

            let mut buffer = String::new();
            f.read_to_string(&mut buffer).unwrap();

            assert_eq!("Example Text!", buffer);
            env::set_var(RUNFILES_DIR_ENV_VAR, &runfiles_dir)
        }

        // Test case 3: Neither are set
        {
            env::remove_var(RUNFILES_DIR_ENV_VAR);
            env::remove_var(TEST_SRCDIR_ENV_VAR);

            let r = Runfiles::create().unwrap();

            let mut f =
                File::open(r.rlocation("rules_rust/tools/runfiles/data/sample.txt")).unwrap();

            let mut buffer = String::new();
            f.read_to_string(&mut buffer).unwrap();

            assert_eq!("Example Text!", buffer);

            env::set_var(TEST_SRCDIR_ENV_VAR, &test_srcdir);
            env::set_var(RUNFILES_DIR_ENV_VAR, &runfiles_dir);
        }

        // Because rules_rust currently doesn't work with modules, we need to pretend that we're
        // using rules_rust in a repo that has bzlmod enabled.
        env::set_current_dir(
            Runfiles::create()
                .unwrap()
                .rlocation("rules_rust/tools/runfiles/data"),
        )
        .unwrap();
        let module_runfiles = [
            {
                let r = Runfiles::create_directory_based(PathBuf::from("bzlmod")).unwrap();
                ("directory", r)
            },
            {
                let r = Runfiles::create_manifest_based(PathBuf::from("bzlmod_manifest")).unwrap();
                ("manifest", r)
            },
        ];

        for (name, r) in module_runfiles {
            // TODO: we are unable to change what current_repository() returns
            #[cfg(not(bazel_root_module))]
            {
                assert_eq!(
                    r.rlocation("tinyjson/sample.txt"),
                    PathBuf::from("bzlmod/rules_rust~tinyjson/sample.txt"),
                    "{name}",
                )
            }
            #[cfg(bazel_root_module)]
            {
                assert_eq!(
                    r.rlocation("tinyjson/sample.txt"),
                    PathBuf::from("bzlmod/my_module~tinyjson/sample.txt"),
                    "{name}",
                );
                assert_eq!(
                    r.rlocation("my_module/sample.txt"),
                    PathBuf::from("bzlmod/_main/sample.txt"),
                    "{name}",
                );
            }
        }
    }

    #[test]
    fn test_manifest_based_can_read_data_from_runfiles() {
        let mut path_mapping = HashMap::new();
        path_mapping.insert("a/b".into(), "c/d".into());
        let r = Runfiles {
            mode: Mode::ManifestBased(path_mapping),
            repo_mapping: None,
        };

        assert_eq!(r.rlocation("a/b"), PathBuf::from("c/d"));
    }

    #[test]
    fn test_current_repository() {
        let r = Runfiles::create().unwrap();

        // This check is unique to the rules_rust repository. The name
        // here is expected to be different in consumers of this library
        #[cfg(not(bazel_root_module))]
        assert_eq!(r.current_repository(), "rules_rust");

        #[cfg(bazel_root_module)]
        assert_eq!(r.current_repository(), "_main");
    }
}
