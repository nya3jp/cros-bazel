use anyhow::{Context, Result};
use std::fs::{remove_dir_all, set_permissions, Permissions};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use walkdir::WalkDir;

const ORWX: u32 = 0o700;

// remove_dir_all_with_chmod calls remove_dir_all after ensuring we have o+rwx to each directory so
// that we can remove all files.
pub fn remove_dir_all_with_chmod<P: AsRef<Path>>(path: P) -> Result<()> {
    for entry in WalkDir::new(&path)
        .into_iter()
        // walk isn't lazy, so if we have a directory with no permissions, it attempts to list its
        // contents (which fails since it has no permissions), then sets permissions.
        // Thus, we filter out any failures in the listing directory stage.
        // If it's an issue, it'll end up being picked up by remove_dir_all anyway.
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let mode = entry.metadata()?.mode();
        if mode & ORWX != ORWX {
            let new_mode = mode | ORWX;
            set_permissions(entry.path(), Permissions::from_mode(new_mode)).with_context(|| {
                format!(
                    "Failed to set permissions for {:?} to {:o}",
                    path.as_ref(),
                    new_mode
                )
            })?;
        }
    }

    remove_dir_all(&path).with_context(|| format!("Failed to delete {:?}", path.as_ref()))
}
