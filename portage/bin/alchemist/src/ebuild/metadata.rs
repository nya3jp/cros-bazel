// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::repository::{Repository, UnorderedRepositorySet};

use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;
use std::{
    ffi::OsStr,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::Command,
};
use version::Version;

use crate::{
    bash::vars::{parse_set_output, BashVars},
    data::Vars,
};

fn run_ebuild<'a>(
    ebuild_path: &Path,
    env: &Vars,
    eclass_dirs: impl IntoIterator<Item = &'a Path>,
    tools_dir: &Path,
) -> Result<BashVars> {
    let mut script_file = tempfile::tempfile()?;
    script_file.write_all(include_bytes!("ebuild_prelude.sh"))?;
    script_file.seek(SeekFrom::Start(0))?;

    let mut set_output_file = tempfile::NamedTempFile::new()?;

    let output = Command::new("/bin/bash")
        .stdin(script_file)
        .current_dir("/")
        .env_clear()
        .envs(env)
        .env("PATH", tools_dir.to_string_lossy().as_ref())
        .env("__xbuild_in_ebuild", ebuild_path.to_string_lossy().as_ref())
        .env(
            "__xbuild_in_eclass_dirs",
            eclass_dirs
                .into_iter()
                .map(|path| format!("{}\n", path.to_string_lossy()))
                .join(""),
        )
        .env(
            "__xbuild_in_output_vars",
            set_output_file.as_ref().to_string_lossy().as_ref(),
        )
        .output()
        .context("Failed to spawn bash for ebuild metadata evaluation")?;

    if !output.status.success() {
        bail!(
            "ebuild failed to evaluate {}: {}\nstdout: {}\nstderr: {}",
            ebuild_path.display(),
            &output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    if !output.stdout.is_empty() || !output.stderr.is_empty() {
        bail!(
            "ebuild printed errors to stdout/stderr\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let mut set_output = String::new();
    set_output_file
        .as_file_mut()
        .read_to_string(&mut set_output)?;
    parse_set_output(&set_output)
}

#[derive(Debug)]
pub(super) struct EBuildEvaluator {
    tools_dir: PathBuf,
}

impl EBuildEvaluator {
    pub(super) fn new(tools_dir: &Path) -> Self {
        Self {
            tools_dir: tools_dir.to_owned(),
        }
    }

    pub(super) fn evaluate_metadata(
        &self,
        ebuild_path: &Path,
        repo: &Repository,
    ) -> Result<MaybeEBuildMetadata> {
        // We don't need to provide profile variables to the ebuild environment
        // because PMS requires ebuild metadata to be defined independently of
        // profiles.
        // https://projects.gentoo.org/pms/8/pms.html#x1-600007.1
        let path_info = EBuildPathInfo::try_from(ebuild_path)?;
        let env = path_info.to_vars();
        let basic_data = EBuildBasicData {
            repo_name: repo.name().to_string(),
            ebuild_path: ebuild_path.to_path_buf(),
            package_name: format!(
                "{}/{}",
                &path_info.category_name, &path_info.short_package_name
            ),
            short_package_name: path_info.short_package_name,
            category_name: path_info.category_name,
            version: path_info.version,
        };
        match run_ebuild(ebuild_path, &env, repo.eclass_dirs(), &self.tools_dir) {
            Ok(vars) => Ok(MaybeEBuildMetadata::Ok(Arc::new(EBuildMetadata {
                basic_data,
                vars,
            }))),
            Err(err) => Ok(MaybeEBuildMetadata::Err(Arc::new(EBuildEvaluationError {
                basic_data,
                error: err.to_string(),
            }))),
        }
    }
}

/// Contains basic information about an ebuild.
///
/// This information is available as long as an ebuild file exists with a correct file name format.
/// All package-representing types containing [`EBuildBasicData`] directly or indirectly should
/// implement [`Deref`] to provide easy access to [`EBuildBasicData`] fields.
#[derive(Debug, Eq, PartialEq)]
pub struct EBuildBasicData {
    pub repo_name: String,
    pub ebuild_path: PathBuf,
    pub package_name: String,
    pub short_package_name: String,
    pub category_name: String,
    pub version: Version,
}

/// Describes metadata of an ebuild.
#[derive(Debug, Eq, PartialEq)]
pub struct EBuildMetadata {
    pub basic_data: EBuildBasicData,
    pub vars: BashVars,
}

impl Deref for EBuildMetadata {
    type Target = EBuildBasicData;

    fn deref(&self) -> &Self::Target {
        &self.basic_data
    }
}

/// Describes an error on evaluating an ebuild.
#[derive(Debug, Eq, PartialEq)]
pub struct EBuildEvaluationError {
    pub basic_data: EBuildBasicData,
    pub error: String,
}

impl Deref for EBuildEvaluationError {
    type Target = EBuildBasicData;

    fn deref(&self) -> &Self::Target {
        &self.basic_data
    }
}

/// Represents an ebuild, covering both successfully evaluated ones and failed ones.
///
/// Since this enum is very lightweight (contains [`Arc`] only), you should not wrap it within
/// reference-counting smart pointers like [`Arc`], but you can just clone it.
///
/// While this enum looks very similar to [`Result`], we don't make it a type alias of [`Result`]
/// to implement a few convenient methods.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeEBuildMetadata {
    Ok(Arc<EBuildMetadata>),
    Err(Arc<EBuildEvaluationError>),
}

impl Deref for MaybeEBuildMetadata {
    type Target = EBuildBasicData;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeEBuildMetadata::Ok(metadata) => &metadata.basic_data,
            MaybeEBuildMetadata::Err(error) => &error.basic_data,
        }
    }
}

/// A bundle of information that can be extracted from an ebuild file path.
#[derive(Debug)]
pub struct EBuildPathInfo {
    pub short_package_name: String,
    pub category_name: String,
    pub version: Version,
}

impl EBuildPathInfo {
    /// Computes an initial ebuild environment derived from an ebuild file path.
    fn to_vars(&self) -> Vars {
        Vars::from_iter(
            [
                (
                    "P",
                    format!(
                        "{}-{}",
                        &self.short_package_name,
                        self.version.without_revision()
                    ),
                ),
                (
                    "PF",
                    format!("{}-{}", &self.short_package_name, self.version),
                ),
                ("PN", self.short_package_name.to_owned()),
                ("CATEGORY", self.category_name.to_owned()),
                ("PV", self.version.without_revision().to_string()),
                ("PR", format!("r{}", self.version.revision())),
                ("PVR", self.version.to_string()),
            ]
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value)),
        )
    }
}

impl TryFrom<&Path> for EBuildPathInfo {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self> {
        if path.extension() != Some(OsStr::new("ebuild")) {
            bail!("{} is not an ebuild file", path.to_string_lossy());
        }

        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let (short_package_name, version) = Version::from_str_suffix(file_stem.as_ref())
            .with_context(|| format!("{} has corrupted file name", path.to_string_lossy()))?;

        let (short_package_name_from_dir, category_name) = path
            .components()
            .rev()
            .skip(1)
            .next_tuple()
            .ok_or_else(|| {
                anyhow!(
                    "{} does not contain necessary directory part",
                    path.to_string_lossy()
                )
            })?;

        if short_package_name != short_package_name_from_dir.as_os_str().to_string_lossy() {
            bail!(
                "{} has inconsistent package names in directory name and file name",
                path.to_string_lossy()
            );
        }

        Ok(Self {
            short_package_name: short_package_name.to_owned(),
            category_name: category_name.as_os_str().to_string_lossy().to_string(),
            version,
        })
    }
}

/// Wraps EBuildEvaluator to cache results.
#[derive(Debug)]
pub struct CachedEBuildEvaluator {
    repos: UnorderedRepositorySet,
    evaluator: EBuildEvaluator,
    cache: Mutex<HashMap<PathBuf, Arc<OnceCell<MaybeEBuildMetadata>>>>,
}

impl CachedEBuildEvaluator {
    pub fn new(repos: UnorderedRepositorySet, tools_dir: &Path) -> Self {
        let evaluator = EBuildEvaluator::new(tools_dir);

        Self {
            repos,
            evaluator,
            cache: Default::default(),
        }
    }

    pub fn evaluate_metadata(&self, ebuild_path: &Path) -> Result<MaybeEBuildMetadata> {
        let once_cell = {
            let mut cache_guard = self.cache.lock().unwrap();
            cache_guard
                .entry(ebuild_path.to_owned())
                .or_default()
                .clone()
        };
        let details = once_cell.get_or_try_init(|| {
            let repo = self.repos.get_repo_by_path(ebuild_path)?;
            self.evaluator.evaluate_metadata(ebuild_path, repo)
        })?;
        Ok(details.clone())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    /// Ensures [`EBuildEvaluator`] successfully evaluates variables of a minimal ebuild.
    #[test]
    fn test_evaluate_vars() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.path();

        let ebuild_path = temp_dir.join("sys-apps/hello/hello-1.2.3.ebuild");
        std::fs::create_dir_all(ebuild_path.parent().unwrap())?;
        std::fs::write(
            &ebuild_path,
            r#"
EAPI=7
SLOT=0
KEYWORDS="*"

WORLD="world"

SCALAR="hello, ${WORLD}!"
ARRAY=(hello, "${WORLD}!")
"#,
        )?;

        let evaluator = EBuildEvaluator::new(&temp_dir.join("tools"));
        let repo = Repository::new_for_testing("test", temp_dir);

        let metadata = evaluator.evaluate_metadata(&ebuild_path, &repo)?;

        let metadata = match metadata {
            MaybeEBuildMetadata::Ok(metadata) => metadata,
            MaybeEBuildMetadata::Err(error) => panic!("Failed to evaluate metadata: {error:?}"),
        };

        // Verify basic data.
        assert_eq!(
            metadata.basic_data,
            EBuildBasicData {
                repo_name: "test".into(),
                ebuild_path,
                package_name: "sys-apps/hello".into(),
                short_package_name: "hello".into(),
                category_name: "sys-apps".into(),
                version: Version::try_new("1.2.3").unwrap(),
            }
        );

        // Vars may contain many unrelated variables, so inspect partially.
        let vars = &metadata.vars;
        assert_eq!(vars.get_scalar("EAPI").unwrap(), "7");
        assert_eq!(vars.get_scalar("SLOT").unwrap(), "0");
        assert_eq!(vars.get_scalar("KEYWORDS").unwrap(), "*");
        assert_eq!(vars.get_scalar("SCALAR").unwrap(), "hello, world!");
        assert_eq!(
            vars.get_indexed_array("ARRAY").unwrap(),
            ["hello,", "world!"]
        );

        Ok(())
    }

    /// Ensures [`EBuildEvaluator`] correctly handles evaluation errors.
    #[test]
    fn test_evaluate_die() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.path();

        let ebuild_path = temp_dir.join("sys-apps/hello/hello-1.2.3.ebuild");
        std::fs::create_dir_all(ebuild_path.parent().unwrap())?;
        std::fs::write(
            &ebuild_path,
            r#"
EAPI=7
SLOT=0
KEYWORDS="*"

die "failed failed failed"
"#,
        )?;

        let evaluator = EBuildEvaluator::new(&temp_dir.join("tools"));
        let repo = Repository::new_for_testing("test", temp_dir);

        // evaluate_metadata should return success even with evaluation errors.
        let metadata = evaluator.evaluate_metadata(&ebuild_path, &repo)?;

        match &metadata {
            MaybeEBuildMetadata::Err(error) if error.error.contains("failed failed failed") => {}
            _ => panic!(
                "Unexpected return value from evaluate_metadata: {:?}",
                metadata
            ),
        }

        Ok(())
    }

    /// Ensures [`EBuildEvaluator`] correctly sets up the ebuild environment.
    #[test]
    fn test_evaluate_ebuild_env() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.path();

        let ebuild_path = temp_dir.join("sys-apps/hello/hello-1.2.3-r99.ebuild");
        std::fs::create_dir_all(ebuild_path.parent().unwrap())?;
        std::fs::write(
            &ebuild_path,
            r#"
EAPI=7
SLOT=0
KEYWORDS="*"

assert_var() {
    local name="$1"
    local got="${!name}"
    local want="$2"
    if [[ "${got}" != "${want}" ]]; then
        die "\$$name = ${got@Q}; want ${want@Q}"
    fi
}

assert_var P "hello-1.2.3"
assert_var PF "hello-1.2.3-r99"
assert_var PN "hello"
assert_var CATEGORY "sys-apps"
assert_var PV "1.2.3"
assert_var PR "r99"
assert_var PVR "1.2.3-r99"
"#,
        )?;

        let evaluator = EBuildEvaluator::new(&temp_dir.join("tools"));
        let repo = Repository::new_for_testing("test", temp_dir);

        let metadata = evaluator.evaluate_metadata(&ebuild_path, &repo)?;
        assert!(
            matches!(metadata, MaybeEBuildMetadata::Ok(_)),
            "metadata = {:?}",
            metadata
        );

        Ok(())
    }

    /// Ensures [`EBuildEvaluator`] returns a fatal error if a specified ebuild path does not follow
    /// the naming convention.
    #[test]
    fn test_evaluate_wrong_ebuild_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.path();

        let evaluator = EBuildEvaluator::new(&temp_dir.join("tools"));
        let repo = Repository::new_for_testing("test", temp_dir);

        let try_evaluate_metadata = |relative_path: &str| {
            let ebuild_path = temp_dir.join(relative_path);
            std::fs::create_dir_all(ebuild_path.parent().unwrap())?;
            std::fs::write(&ebuild_path, "EAPI=7\nSLOT=0\nKEYWORDS='*'\n")?;
            evaluator.evaluate_metadata(&ebuild_path, &repo)
        };

        try_evaluate_metadata("sys-apps/hello/hello-1.2.3.ebuild").expect("Must succeed");
        // Wrong extension.
        try_evaluate_metadata("sys-apps/hello/hello-1.2.3.eclass")
            .expect_err("Must fail for wrong extension");
        // Wrong version number.
        try_evaluate_metadata("sys-apps/hello/hello-1.2.3_foobar.ebuild")
            .expect_err("Must fail for wrong version number");
        // The directory name and the ebuild name does not match.
        try_evaluate_metadata("sys-apps/hello/world-1.2.3.ebuild")
            .expect_err("Must fail for inconsistent file names");

        Ok(())
    }
}
