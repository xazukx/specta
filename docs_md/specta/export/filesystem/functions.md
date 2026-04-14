# Functions in specta::export::filesystem

```rust
/**
`cleanup_stale_files` -- Delete stale generated files and clean up empty directories.

A file is considered stale if:
1. It has one of the given extensions
2. It contains the generated marker
3. It is not in the `current_files` map
*/
pub fn cleanup_stale_files(root: &std::path::Path, current_files: &std::collections::HashMap<std::path::PathBuf, String>, extensions: &[&str], marker: &str) -> Result<(), io::Error>
/**
`collect_existing_files` -- Collect all files with the given extensions in a directory recursively.
*/
pub fn collect_existing_files(root: &std::path::Path, extensions: &[&str]) -> Result<std::collections::HashSet<std::path::PathBuf>, io::Error>
/**
`is_generated_file` -- Check whether a file contains the given generated marker string.
*/
pub fn is_generated_file(path: &std::path::Path, marker: &str) -> Result<bool, io::Error>
/**
`remove_empty_dirs` -- Remove empty directories recursively, stopping at the root.
*/
pub fn remove_empty_dirs(path: &std::path::Path, root: &std::path::Path) -> Result<(), io::Error>
```

