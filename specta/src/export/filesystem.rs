use std::{
    collections::{BTreeMap, HashSet},
    io,
    path::{Path, PathBuf},
};

/// Collect all files with the given extensions in a directory recursively.
pub fn collect_existing_files(
    root: &Path,
    extensions: &[&str],
) -> Result<HashSet<PathBuf>, io::Error> {
    if !root.exists() {
        return Ok(HashSet::new());
    }

    let mut files = HashSet::new();
    let entries = std::fs::read_dir(root)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            files.extend(collect_existing_files(&path, extensions)?);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.iter().any(|e| *e == ext) {
                files.insert(path);
            }
        }
    }

    Ok(files)
}

/// Check whether a file contains the given generated marker string.
pub fn is_generated_file(path: &Path, marker: &str) -> Result<bool, io::Error> {
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(contents.contains(marker)),
        Err(err) if err.kind() == io::ErrorKind::InvalidData => Ok(false),
        Err(err) => Err(err),
    }
}

/// Remove empty directories recursively, stopping at the root.
pub fn remove_empty_dirs(path: &Path, root: &Path) -> Result<(), io::Error> {
    let entries = std::fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            remove_empty_dirs(&entry_path, root)?;
        }
    }

    let is_empty = path.read_dir()?.next().is_none();

    if path != root && is_empty {
        match std::fs::remove_dir(path) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

/// Delete stale generated files and clean up empty directories.
///
/// A file is considered stale if:
/// 1. It has one of the given extensions
/// 2. It contains the generated marker
/// 3. It is not in the `current_files` map
pub fn cleanup_stale_files(
    root: &Path,
    current_files: &BTreeMap<PathBuf, String>,
    extensions: &[&str],
    marker: &str,
) -> Result<(), io::Error> {
    for path in collect_existing_files(root, extensions)? {
        if current_files.contains_key(&path) || !is_generated_file(&path, marker)? {
            continue;
        }

        std::fs::remove_file(&path).or_else(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(err)
            }
        })?;
    }

    remove_empty_dirs(root, root)?;

    Ok(())
}
