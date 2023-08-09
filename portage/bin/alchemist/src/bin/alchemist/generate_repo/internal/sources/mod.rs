// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    collections::{BTreeMap, HashSet},
    fs::{create_dir_all, read_link, File},
    io::{ErrorKind, Write},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use crate::generate_repo::common::{escape_starlark_string, AUTOGENERATE_NOTICE};
use alchemist::analyze::source::PackageLocalSource;
use anyhow::{Context, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use tera::Tera;
use tracing::instrument;
use walkdir::WalkDir;

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera: Tera = Default::default();
        tera.add_raw_template(
            "source.BUILD.bazel",
            include_str!("templates/source.BUILD.bazel"),
        )
        .unwrap();
        tera.autoescape_on(vec![".bazel"]);
        tera.set_escape_fn(escape_starlark_string);
        tera
    };
}

const VALID_TARGET_NAME_PUNCTUATIONS: &str = r##"!%-@^_"#$&'()*-+,;<=>?[]{|}~/."##;

/// Checks if a character is allowed in Bazel target names.
fn is_valid_target_name_char(c: char) -> bool {
    // See https://bazel.build/concepts/labels?hl=en#target-names for the
    // lexical specification of valid target names.
    c.is_ascii_alphanumeric() || VALID_TARGET_NAME_PUNCTUATIONS.contains(c)
}

/// Checks if a string is a valid Bazel target name.
fn is_valid_target_name(s: &str) -> bool {
    s.chars().all(is_valid_target_name_char)
}

/// Escapes a file name string so that it can be used as a Bazel target name.
///
/// It doesn't take care of invalid file names, such as:
/// - empty string
/// - strings starting with ./ or ../
fn escape_file_name_for_bazel_target_name(s: &str) -> String {
    s.bytes()
        .map(|b| {
            let c = b as char;
            if is_valid_target_name_char(c) && c != '%' {
                String::from(c)
            } else {
                format!("%{:02X}", b)
            }
        })
        .join("")
}

// These files also need to be excluded in `source.BUILD.bazel`.
const BAZEL_SPECIAL_FILE_NAMES: &[&str] = &["BUILD.bazel", "BUILD", "WORKSPACE.bazel", "WORKSPACE"];

/// Checks if a file name needs renaming.
fn file_path_needs_renaming(path: &Path) -> bool {
    if !is_valid_target_name(&*path.to_string_lossy()) {
        return true;
    }

    let file_name = &*path.file_name().unwrap_or_default().to_string_lossy();
    BAZEL_SPECIAL_FILE_NAMES.contains(&file_name)
}

/// Describes the layout of a local source package.
///
/// A local source package corresponds to a directory in ChromeOS source code
/// checkout that appears in `CROS_WORKON_*` variables of any valid .ebuild
/// file. This struct describes its location and children.
///
/// This struct contains information computed sorely from ebuild metadata.
/// It is eventually turned into [`SourcePackage`] after collecting more
/// information from other sources, such as the file system.
#[derive(Clone, Debug)]
struct SourcePackageLayout {
    /// Relative directory path from the "src" directory of ChromeOS checkout.
    /// e.g. "platform2/debugd".
    prefix: PathBuf,

    /// Relative directory paths of child local source packages.
    /// e.g. "platform2/debugd/dbus_bindings".
    child_prefixes: Vec<PathBuf>,
}

impl SourcePackageLayout {
    /// Computes a list of [`SourcePackageLayout`] from a list of [`PackageLocalSource`].
    fn compute<'a>(
        all_local_sources: impl IntoIterator<Item = &'a PackageLocalSource>,
    ) -> Result<Vec<Self>> {
        // Deduplicate all local source prefixes.
        let sorted_prefixes: Vec<PathBuf> = all_local_sources
            .into_iter()
            .filter_map(|origin| match origin {
                PackageLocalSource::Src(src) => Some(PathBuf::from(src)),
                _ => None,
            })
            .sorted()
            .dedup()
            .collect();

        // Create initial empty SourcePackageLayout's.
        let mut layout_map: BTreeMap<PathBuf, SourcePackageLayout> =
            BTreeMap::from_iter(sorted_prefixes.iter().map(|prefix| {
                (
                    prefix.clone(),
                    SourcePackageLayout {
                        prefix: prefix.clone(),
                        child_prefixes: Vec::new(),
                    },
                )
            }));

        // Compute children of each local source package.
        let mut stack: Vec<&Path> = Vec::new();
        for prefix in sorted_prefixes.iter() {
            while let Some(parent_prefix) = stack.pop() {
                if prefix.starts_with(parent_prefix) {
                    // `prefix` is a child of `parent_prefix`.
                    layout_map
                        .get_mut(parent_prefix)
                        .unwrap()
                        .child_prefixes
                        .push(prefix.to_owned());
                    stack.push(parent_prefix);
                    break;
                }
            }
            stack.push(prefix);
        }

        Ok(layout_map.into_values().collect())
    }
}

/// Describes a file to rename on generating a source package.
///
/// We need to rename a file when its path contains special characters not
/// allowed in Bazel target names, or it is treated specially by Bazel (e.g.
/// `BUILD.bazel`). This struct describes how to handle such a file.
#[derive(Clone, Debug)]
struct Rename {
    /// The original file path, relative from the source package directory.
    source_path: PathBuf,
    /// The new file path, relative from the output directory.
    output_path: PathBuf,
}

/// Represents a local source package.
///
/// This struct is similar to [`SourcePackageLayout`], but it contains
/// information gathered by scanning the actual source code directory.
#[derive(Clone, Debug)]
struct SourcePackage {
    /// [`SourcePackageLayout`] describing the layout of this local source
    /// package.
    layout: SourcePackageLayout,

    /// Absolute path to the source directory.
    source_dir: PathBuf,

    /// Absolute path to the output directory.
    output_dir: PathBuf,

    // Directory paths relative from the source package directory.
    dirs: Vec<PathBuf>,

    /// File paths, relative from the source package directory, of symbolic
    /// links under the source package.
    symlinks: Vec<PathBuf>,

    /// File paths to rename on generating a source package. This is necessary
    /// to handle file paths containing special characters not allowed in Bazel
    /// target names.
    renames: Vec<Rename>,

    /// File paths, relative from the source package directory, of files to
    /// exclude from globbing in the source package.
    /// For example, `BUILD.bazel` files existing in the source directories
    /// should be excluded to avoid confusing Bazel, thus they appear here.
    /// `excludes` includes `symlinks` and `renames`.
    excludes: Vec<PathBuf>,

    /// Whether to include .git in the source tarball.
    /// This must not be enabled except for a few packages under active fixing
    /// because .git structure is not canonicalized and thus harms caching.
    /// TODO: Remove this hack.
    include_git: bool,
}

impl SourcePackage {
    /// Computes [`SourcePackage`] from [`SourcePackageLayout`] and the result of
    /// scanning the directory.
    #[instrument(skip_all, fields(prefix = %layout.prefix.display()))]
    fn try_new(
        layout: SourcePackageLayout,
        src_dir: &Path,
        repository_output_dir: &Path,
    ) -> Result<Self> {
        let source_dir = src_dir.join(&layout.prefix);

        let output_dir = repository_output_dir
            .join("internal/sources")
            .join(&layout.prefix);

        // Pre-compute child package paths for fast lookup.
        let child_paths: HashSet<PathBuf> = layout
            .child_prefixes
            .iter()
            .map(|child_prefix| src_dir.join(child_prefix))
            .collect();

        // Find files to handle specially.
        let mut dirs = Vec::new();
        let mut symlinks = Vec::new();
        let mut renames = Vec::new();
        let mut excludes = Vec::new();
        let mut walk = WalkDir::new(&source_dir)
            .sort_by_file_name()
            .min_depth(1)
            .into_iter()
            .filter_entry(|entry| !child_paths.contains(entry.path()));
        // We cannot use "for ... in" here because WalkDir::skip_current_dir
        // needs a mutable borrow.
        loop {
            let entry = match walk.next() {
                None => break,
                Some(entry) => entry?,
            };

            let rel_path = entry.path().strip_prefix(&source_dir)?.to_owned();
            let file_name_str = &*rel_path.file_name().unwrap_or_default().to_string_lossy();

            // Skip .git directory. Note that this is also hard-coded in
            // the template BUILD.bazel.
            if file_name_str == ".git" {
                // Note that .git can be a symlink, in which case we must not
                // call WalkDir::skip_current_dir.
                if entry.file_type().is_dir() {
                    walk.skip_current_dir();
                }
                continue;
            }

            // Record directories.
            if entry.file_type().is_dir() {
                dirs.push(rel_path);
                continue;
            }

            // Record symlinks.
            if entry.file_type().is_symlink() {
                symlinks.push(rel_path.clone());
                excludes.push(rel_path);
                continue;
            }

            // Record files that need renaming.
            // We can ignore directories since they're not matched by glob().
            // This means that empty directories are not reproduced, but Git
            // doesn't support them anyway.
            if file_path_needs_renaming(&rel_path) {
                renames.push(Rename {
                    source_path: rel_path.clone(),
                    output_path: PathBuf::from(format!("__rename__{}", renames.len())),
                });
                excludes.push(rel_path);
                continue;
            }
        }

        // HACK: We intentionally include the .git repo for llvm because it's
        // required to calculate which patches to apply. We really need to
        // figure out another way of doing this.
        // TODO: Remove this hack.
        let include_git = layout.prefix.to_string_lossy() == "third_party/llvm-project";

        if include_git {
            // On including .git, explicitly scan directories as it may contain
            // mandatory empty directories (e.g. .git/refs). We don't bother to
            // handle file names with special characters under .git; if such
            // files exist, they will just cause the build to fail.
            let walk = WalkDir::new(source_dir.join(".git"))
                .follow_links(true)
                .sort_by_file_name();
            for entry in walk {
                let entry = entry?;
                if entry.file_type().is_dir() {
                    let rel_path = entry.path().strip_prefix(&source_dir)?.to_owned();
                    dirs.push(rel_path);
                }
            }
        }

        Ok(Self {
            layout,
            source_dir,
            output_dir,
            dirs,
            symlinks,
            renames,
            excludes,
            include_git,
        })
    }
}

#[derive(Serialize)]
struct SymlinkEntry {
    name: String,
    location: String,
    target: String,
}

#[derive(Serialize)]
struct RenameEntry {
    source_path: String,
    output_path: String,
}

#[derive(Serialize)]
struct BuildTemplateContext {
    prefix: String,
    children: Vec<String>,
    dirs: Vec<String>,
    symlinks: Vec<SymlinkEntry>,
    renames: Vec<RenameEntry>,
    excludes: Vec<String>,
    include_git: bool,
}

/// Generates `BUILD.bazel` file for a source package.
fn generate_build_file(package: &SourcePackage) -> Result<()> {
    let context = BuildTemplateContext {
        prefix: package.layout.prefix.to_string_lossy().into_owned(),
        children: package
            .layout
            .child_prefixes
            .iter()
            .map(|prefix| prefix.to_string_lossy().into_owned())
            .collect(),
        dirs: package
            .dirs
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect(),
        symlinks: package
            .symlinks
            .iter()
            .map(|path| {
                let location = path.to_string_lossy().into_owned();
                let name = format!(
                    "__symlink__{}",
                    escape_file_name_for_bazel_target_name(&location)
                );
                let target = read_link(package.source_dir.join(path))?
                    .to_string_lossy()
                    .into_owned();
                Ok(SymlinkEntry {
                    name,
                    location,
                    target,
                })
            })
            .collect::<Result<_>>()?,
        renames: package
            .renames
            .iter()
            .map(|rename| RenameEntry {
                source_path: rename.source_path.to_string_lossy().into_owned(),
                output_path: rename.output_path.to_string_lossy().into_owned(),
            })
            .collect(),
        excludes: package
            .excludes
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect(),
        include_git: package.include_git,
    };

    let mut file = File::create(package.output_dir.join("BUILD.bazel"))?;
    file.write_all(AUTOGENERATE_NOTICE.as_bytes())?;
    TEMPLATES.render_to(
        "source.BUILD.bazel",
        &tera::Context::from_serialize(context)?,
        file,
    )?;

    Ok(())
}

/// Generates renaming symlinks for a source package.
///
/// Renaming symlinks are used to deal with files with special names, e.g.
/// containing characters not allowed in Bazel target names or Bazel's special
/// file names such as `BUILD`. We create symlinks to those files with "safe"
/// file names, and use the "rename" attribute in rules_pkg to rename on
/// creating archives.
fn generate_renaming_symlinks(package: &SourcePackage) -> Result<()> {
    for rename in package.renames.iter() {
        let source_path = package.source_dir.join(&rename.source_path);
        let output_path = package.output_dir.join(&rename.output_path);
        symlink(source_path, output_path)?;
    }

    Ok(())
}

/// Generates general symlinks for a source package.
///
/// We replicate the original source directory structure by generating symlinks
/// with the same name as original files. We call them *general* symlinks to
/// distinguish them from *renaming* symlinks.
///
/// This function tries to minimize the number of directories and general
/// symlinks in the output. If we can use a source directory as-is (e.g. no need
/// to generate `BUILD.bazel`), we create a symlink to the whole directory,
/// rather than creating a directory and a bunch of symlinks underneath it.
fn generate_general_symlinks(package: &SourcePackage) -> Result<()> {
    // Create child source package directories in case they have not been
    // created yet.
    for child_prefix in package.layout.child_prefixes.iter() {
        let child_path = package
            .output_dir
            .join(child_prefix.strip_prefix(&package.layout.prefix).unwrap());
        create_dir_all(&child_path)?;
    }

    // Create parent directories of excluded files so that we don't create
    // symlinks for their whole parent directories.
    for rel_path in package.excludes.iter() {
        let dir_path = package
            .output_dir
            .join(rel_path)
            .parent()
            .unwrap()
            .to_owned();
        create_dir_all(&dir_path)?;
    }

    // Create parent directories of files/directories whose name containing
    // special characters not allowed in Bazel target names.
    for rename in package.renames.iter() {
        // rename.source_path points to a regular file, so skip the first entry.
        for rel_path in rename.source_path.ancestors().skip(1usize) {
            if is_valid_target_name(&*rel_path.to_string_lossy()) {
                create_dir_all(package.output_dir.join(rel_path))?;
                break;
            }
        }
    }

    // Precompute hashsets for fast lookup.
    let children_set: HashSet<PathBuf> = package
        .layout
        .child_prefixes
        .iter()
        .map(|child_prefix| {
            // Children paths relative to the current package.
            // Example:
            // package.layout.prefix = "platform2/debugd"
            // package.layout.children[0] = "platform2/debugd/dbus_bindings"
            // => "dbus_bindings"
            child_prefix
                .strip_prefix(&package.layout.prefix)
                .unwrap()
                .to_owned()
        })
        .collect();
    let exclude_set: HashSet<&PathBuf> = package.excludes.iter().collect();

    // Walk the *output* tree to create symlinks.
    let walk = WalkDir::new(&package.output_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| entry.file_type().is_dir());
    for output_dir_entry in walk {
        let output_dir_entry = output_dir_entry?;
        let dir_rel_path = output_dir_entry
            .path()
            .strip_prefix(&package.output_dir)
            .unwrap();

        // Do not step into children's directories.
        if children_set.contains(dir_rel_path) {
            continue;
        }

        let source_dir = package.source_dir.join(dir_rel_path);
        let read_dir = source_dir
            .read_dir()
            .with_context(|| format!("Failed to read directory {:?}", &source_dir))?;
        for source_file_entry in read_dir {
            let source_file_entry = source_file_entry?;
            let file_rel_path = dir_rel_path.join(source_file_entry.file_name());

            // Skip file paths invalid as Bazel target names. They are processed
            // in generate_renaming_symlinks.
            if !is_valid_target_name(&*file_rel_path.to_string_lossy()) {
                continue;
            }

            // Skip creating symlinks for excluded files.
            if exclude_set.contains(&file_rel_path) {
                continue;
            }

            // TODO: Use relative symlinks so renaming the top directory doesn't
            // invalidate symlinks.
            let source_file_path = source_file_entry.path();
            let output_file_path = output_dir_entry.path().join(source_file_entry.file_name());
            match symlink(&source_file_path, &output_file_path) {
                // EEXIST is OK.
                Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
                other => other.with_context(|| {
                    format!(
                        "Failed to create symlink {:?} -> {:?}",
                        &source_file_path, &output_file_path
                    )
                })?,
            }
        }
    }

    Ok(())
}

/// Generates a source package.
#[instrument(fields(path = %package.source_dir.display()), skip(package))]
fn generate_package(package: &SourcePackage) -> Result<()> {
    create_dir_all(&package.output_dir)?;
    generate_build_file(package)?;
    generate_renaming_symlinks(package)?;
    generate_general_symlinks(package)?;
    Ok(())
}

/// Generates source packages under `@portage//internal/sources/`.
#[instrument(skip_all)]
pub fn generate_internal_sources<'a>(
    all_local_sources: impl IntoIterator<Item = &'a PackageLocalSource>,
    src_dir: &Path,
    repository_output_dir: &Path,
) -> Result<()> {
    let source_layouts: Vec<SourcePackageLayout> = SourcePackageLayout::compute(all_local_sources)?;

    let source_packages: Vec<SourcePackage> = source_layouts
        .into_iter()
        .flat_map(|layout| {
            let prefix_string = layout.prefix.to_string_lossy().into_owned();
            match SourcePackage::try_new(layout, src_dir, repository_output_dir) {
                Ok(source_package) => Some(source_package),
                Err(err) => {
                    eprintln!(
                        "WARNING: Failed to generate source package for {}: {:?}",
                        prefix_string, err
                    );
                    None
                }
            }
        })
        .collect();

    // Generate source packages.
    source_packages.iter().try_for_each(generate_package)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use testutil::compare_with_golden_data;

    use super::*;

    #[test]
    fn test_escape_file_name_for_bazel_target_name() {
        assert_eq!("foo", escape_file_name_for_bazel_target_name("foo"));
        assert_eq!("foo/bar", escape_file_name_for_bazel_target_name("foo/bar"));
        assert_eq!(
            "foo%20bar",
            escape_file_name_for_bazel_target_name("foo bar")
        );
        assert_eq!(
            "foo%25bar",
            escape_file_name_for_bazel_target_name("foo%bar")
        );
        assert_eq!(
            r##"!%25-@^_"#$&'()*-+,;<=>?[]{|}~/."##,
            escape_file_name_for_bazel_target_name(r##"!%-@^_"#$&'()*-+,;<=>?[]{|}~/."##)
        );
        assert_eq!("%F0%9F%90%88", escape_file_name_for_bazel_target_name("🐈"));
    }

    #[test]
    fn test_file_path_needs_renaming() {
        assert!(!file_path_needs_renaming(&PathBuf::from("foo/bar/baz")));
        assert!(!file_path_needs_renaming(&PathBuf::from(
            "foo/bar/BUILD.gn"
        )));
        assert!(file_path_needs_renaming(&PathBuf::from(
            "foo/bar/BUILD.bazel"
        )));
        assert!(file_path_needs_renaming(&PathBuf::from("foo/bar/BUILD")));
        assert!(file_path_needs_renaming(&PathBuf::from("foo/bar/b a z")));
        assert!(file_path_needs_renaming(&PathBuf::from(
            "foo/bar/b a z/hoge"
        )));
    }

    const TESTDATA_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bin/alchemist/generate_repo/internal/sources/testdata"
    );

    /// Tests [`generate_internal_sources`] with scenarios found in `testdata`
    /// directory.
    ///
    /// A directory under `testdata` corresponds to a test case, and it must
    /// contain the following files:
    ///
    /// - `sources.json`: JSON-serialized `Vec<PackageLocalSource>`.
    /// - `source`: Input source directory.
    /// - `golden`: Expected output directory.
    ///
    /// You can regenerate golden directories by running the following command:
    ///
    /// `ALCHEMY_REGENERATE_GOLDEN=1 cargo test`
    #[test]
    fn test_generate_internal_sources_with_golden_data() -> Result<()> {
        let testdata_dir = Path::new(TESTDATA_DIR);

        for entry in testdata_dir.read_dir()? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            eprintln!("Testing {:?}...", entry.file_name());
            let case_dir = entry.path();
            let case_input_path = case_dir.join("sources.json");
            let case_source_dir = case_dir.join("source");
            let case_golden_dir = case_dir.join("golden");

            let local_sources: Vec<PackageLocalSource> = {
                let file = File::open(&case_input_path)?;
                serde_json::from_reader(&file)?
            };

            let temp_dir = tempdir()?;
            let output_dir = temp_dir.path();

            generate_internal_sources(&local_sources, &case_source_dir, output_dir)?;

            let inner_output_dir = output_dir.join("internal/sources");
            compare_with_golden_data(&inner_output_dir, &case_golden_dir)?;
        }
        Ok(())
    }

    /// Tests [`generate_internal_sources`] with empty directories.
    ///
    /// This test case is separated from [`test_generate_internal_sources_with_golden_data`] because
    /// Git cannot track empty directories.
    ///
    /// The golden output for BUILD.bazel is found at `testdata/empty_dirs.golden.BUILD.bazel`.
    ///
    /// You can regenerate golden directories by running the following command:
    ///
    /// `ALCHEMY_REGENERATE_GOLDEN=1 cargo test`
    #[test]
    fn test_generate_internal_sources_with_empty_dirs() -> Result<()> {
        let source_dir = tempdir()?;
        let source_dir = source_dir.path();

        create_dir_all(source_dir.join("empty_dir"))?;
        create_dir_all(source_dir.join("empty   dir"))?;
        create_dir_all(source_dir.join("nested/empty_dir"))?;
        create_dir_all(source_dir.join("nested/empty   dir"))?;

        let output_dir = tempdir()?;
        let output_dir = output_dir.path();

        generate_internal_sources(
            &[PackageLocalSource::Src(PathBuf::from(""))],
            source_dir,
            output_dir,
        )?;

        let output_path = output_dir.join("internal/sources/BUILD.bazel");
        let golden_path = Path::new(TESTDATA_DIR).join("empty_dirs.golden.BUILD.bazel");
        compare_with_golden_data(&output_path, &golden_path)?;

        Ok(())
    }

    /// Tests [`generate_internal_sources`] with .git directory with empty directories.
    ///
    /// This test case is separated from [`test_generate_internal_sources_with_golden_data`] because
    /// Git cannot track empty directories and files named ".git".
    ///
    /// The golden outputs for BUILD.bazel are found at:
    /// - `testdata/empty_dirs_git.llvm-project.golden.BUILD.bazel`
    /// - `testdata/empty_dirs_git.platform2.golden.BUILD.bazel`
    ///
    /// You can regenerate golden directories by running the following command:
    ///
    /// `ALCHEMY_REGENERATE_GOLDEN=1 cargo test`
    #[test]
    fn test_generate_internal_sources_with_empty_dirs_git() -> Result<()> {
        let git_dir = tempdir()?;
        let git_dir = git_dir.path();

        create_dir_all(git_dir.join("refs"))?;
        File::create(git_dir.join("packed-refs"))?;

        let source_dir = tempdir()?;
        let source_dir = source_dir.path();

        // .git is usually a symlink.
        create_dir_all(source_dir.join("third_party/llvm-project"))?;
        symlink(git_dir, source_dir.join("third_party/llvm-project/.git"))?;
        create_dir_all(source_dir.join("platform2"))?;
        symlink(git_dir, source_dir.join("platform2/.git"))?;

        let output_dir = tempdir()?;
        let output_dir = output_dir.path();

        generate_internal_sources(
            &[
                PackageLocalSource::Src(PathBuf::from("third_party/llvm-project")),
                PackageLocalSource::Src(PathBuf::from("platform2")),
            ],
            source_dir,
            output_dir,
        )?;

        // llvm-project's BUILD.bazel contains pkg_mkdirs.
        compare_with_golden_data(
            &output_dir.join("internal/sources/third_party/llvm-project/BUILD.bazel"),
            &Path::new(TESTDATA_DIR).join("empty_dirs_git.llvm-project.golden.BUILD.bazel"),
        )?;

        // platform2's BUILD.bazel excludes .git and does not contain pkg_mkdirs because it's not
        // allowlisted.
        compare_with_golden_data(
            &output_dir.join("internal/sources/platform2/BUILD.bazel"),
            &Path::new(TESTDATA_DIR).join("empty_dirs_git.platform2.golden.BUILD.bazel"),
        )?;

        Ok(())
    }
}
