// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Result};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::iter::Iterator;
use std::path::{Path, PathBuf};

/// Filters paths to only paths that look like paths to shared libraries.
fn filter_shared_lib_filenames<'a>(paths: &[&'a Path]) -> Vec<&'a Path> {
    let file_name_regex = Regex::new(r"^.+\.so(?:\.[0-9]+){0,3}$").unwrap();
    paths
        .iter()
        .cloned()
        .filter(move |p| {
            let file_name = p.file_name().unwrap().to_string_lossy();
            file_name_regex.is_match(&file_name)
        })
        .collect()
}

/// Generates a LD_LIBRARY_PATH-like object.
/// Each entry in directory_regexes is assumed to correspond to an entry in LD_LIBRARY_PATH, but
/// we don't know exactly what the filename is.
/// Thus, we use a set of known paths to approximate it.
/// Eg. `generate_ld_library_path(["/clang/16/libfoo.so"], ["/clang/\d+"]) -> ["/clang/16"])`
pub(crate) fn generate_ld_library_path(
    paths: &[&Path],
    directory_regexes: &[Regex],
) -> Result<Vec<PathBuf>> {
    let paths = filter_shared_lib_filenames(paths);
    let dirs: HashSet<&Path> = paths.iter().map(|p| p.parent().unwrap()).collect();

    // A mapping from index in LD_LIBRARY_PATH to the value.
    // Order matters because the index is like a priority.
    let mut library_paths: BTreeMap<usize, PathBuf> = BTreeMap::new();
    for dir in dirs {
        let dir_str = dir.to_string_lossy();
        for (idx, matcher) in directory_regexes.iter().enumerate() {
            if let Some(m) = matcher.find(&dir_str) {
                // Verify that we matched the whole directory, not just the first part.
                if m.as_str() == dir_str {
                    if let Some(old_dir) = library_paths.insert(idx, dir.to_path_buf()) {
                        bail!(
                            "{:?} matched both {dir:?} and {old_dir:?}. \
                            Each entry must match a single path.",
                            directory_regexes[idx]
                        )
                    }
                    break;
                }
            }
        }
    }
    Ok(library_paths.values().cloned().collect())
}

/// Given several shared libraries of the same name, use LD_LIBRARY_PATH to select which one to use.
/// The rest will be "masked".
/// Eg. `choose_library(
///   ["/lib32/libfoo.so", "/lib64/libfoo.so"],
///   ["/lib64", "/lib32"]
/// ) -> "/lib64/libfoo.so"`
/// since lib64 comes before lib32 in the LD_LIBRARY_PATH.
fn choose_library<'a>(files: &[&'a Path], ld_library_path: &[PathBuf]) -> Option<&'a Path> {
    files
        .iter()
        .flat_map(|path| {
            let got_dir = path.parent().unwrap();
            let priority = ld_library_path
                .iter()
                .position(|want_dir| want_dir == got_dir);
            priority.map(|priority| (priority, path))
        })
        .min()
        .map(|(_, path)| *path)
}

/// Filters a list of files down to only specific shared libraries.
/// Outputs shared libraries in library search path ordering, removing duplicates.
///
/// # Arguments
/// * `paths` - A list of paths that may or may not be shared libraries.
/// * `directory_regexes` - A list of regexes, treated similarly to LD_LIBRARY_PATH.
///     Each entry in this list can only match a single directory.
///
/// # Examples
/// ```calculate_shared_libraries(
///    ["/usr/lib64/libfoo.so", "/lib64/libfoo.so", "/usr/lib64/libbar.so"],
///    [r"/lib\d*", r"/usr/lib\d*"]
///  )```
/// The example above would return:
/// ```SharedLibraries{
///    ld_library_path: ["/lib64", "/usr/lib64"],
///    shared_libraries: ["/lib/libfoo.so", "/usr/lib64/libbar.so"]
///  }```
/// This is because, though /usr/lib64/libfoo.so does match an entry, it is masked by
/// /lib64/libfoo.so similar to how it would be masked by LD_LIBRARY_PATH.
pub(crate) fn calculate_shared_libraries<'a>(
    paths: &[&'a Path],
    ld_library_path: &[PathBuf],
) -> Result<BTreeSet<&'a Path>> {
    let paths = filter_shared_lib_filenames(paths);

    let mut name_to_paths: HashMap<&std::ffi::OsStr, Vec<&'a Path>> = HashMap::new();
    for path in paths {
        let name = path.file_name().unwrap();
        name_to_paths.entry(name).or_default().push(path);
    }
    Ok(name_to_paths
        .values()
        .flat_map(|files| choose_library(files, ld_library_path))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_selection() {
        let lib64 = PathBuf::from("/lib64");
        let lib32 = PathBuf::from("/lib32");

        let lib64_foo = Path::new("/lib64/libfoo.so");
        let lib32_foo = Path::new("/lib32/libfoo.so");
        let unknown_foo = Path::new("/unknown/libfoo.so");

        assert_eq!(
            choose_library(&[&unknown_foo], &[lib64.clone(), lib32.clone()]),
            None
        );
        assert_eq!(
            choose_library(&[&lib64_foo], &[lib64.clone(), lib32.clone()]),
            Some(lib64_foo)
        );

        // The one in the directory corresponding to the first entry should be returned.
        assert_eq!(
            choose_library(&[&lib32_foo, &lib64_foo], &[lib32.clone(), lib64.clone()]),
            Some(lib32_foo)
        );
        assert_eq!(
            choose_library(&[&lib64_foo, &lib32_foo], &[lib32.clone(), lib64.clone()]),
            Some(lib32_foo)
        );

        assert_eq!(
            choose_library(&[&lib32_foo, &lib64_foo], &[lib64.clone(), lib32.clone()]),
            Some(lib64_foo)
        );
        assert_eq!(
            choose_library(&[&lib64_foo, &lib32_foo], &[lib64.clone(), lib32.clone()]),
            Some(lib64_foo)
        );
    }

    #[test]
    fn test_filter_shared_libraries() {
        let valid_paths = [
            Path::new("/path/to/libfoo.so"),
            Path::new("/path/to/libfoo.so.1"),
            Path::new("/path/to/libfoo.so.1.2"),
            Path::new("/path/to/libfoo.so.1.2.3"),
        ];
        assert_eq!(filter_shared_lib_filenames(&valid_paths), &valid_paths);

        let invalid_paths = [Path::new("/path/to/libfoo")];
        assert_eq!(
            filter_shared_lib_filenames(&invalid_paths),
            Vec::<&Path>::new()
        );
    }

    #[test]
    fn test_generate_valid_library_path() -> Result<()> {
        let paths = [
            Path::new("/lib64/libfoo.so"),
            Path::new("/clang/16/libfoo.so"),
            Path::new("/clang/16/bar/libfoo.so"),
        ];
        assert_eq!(
            generate_ld_library_path(
                &paths,
                &[Regex::new("/lib64")?, Regex::new(r"/clang/\d+")?,]
            )?,
            vec![Path::new("/lib64"), Path::new("/clang/16"),]
        );

        assert_eq!(
            generate_ld_library_path(
                &paths,
                &[Regex::new(r"/clang/\d+")?, Regex::new("/lib64")?,]
            )?,
            vec![PathBuf::from("/clang/16"), PathBuf::from("/lib64"),]
        );

        Ok(())
    }

    #[test]
    fn test_generate_duplicate_library_path() -> Result<()> {
        assert_eq!(
            generate_ld_library_path(
                &[
                    Path::new("/clang/16/libfoo.so"),
                    Path::new("/clang/16/libbar.so"),
                ],
                &[Regex::new(r"/clang/\d+")?]
            )?,
            vec![Path::new("/clang/16"),]
        );

        assert!(matches!(
            generate_ld_library_path(
                &[
                    Path::new("/clang/16/libfoo.so"),
                    Path::new("/clang/17/libbar.so"),
                ],
                &[Regex::new(r"/clang/\d+")?]
            ),
            Err(_)
        ));

        Ok(())
    }

    #[test]
    fn test_calculate_shared_libraries() -> Result<()> {
        assert_eq!(
            calculate_shared_libraries(
                &[
                    Path::new("/lib64/libfoo.so"),
                    Path::new("/lib64/foo/libbar.so"),
                ],
                &[PathBuf::from("/lib64")],
            )?,
            [Path::new("/lib64/libfoo.so")].into_iter().collect(),
        );

        Ok(())
    }
}
