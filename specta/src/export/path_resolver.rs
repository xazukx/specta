use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{Constants, Types};

use super::{
    layout::{FolderGrouping, Layout, MultiFileConfig},
    module_graph::{Module, build_module_graph},
};

/// Resolves module paths and named types to filesystem paths.
///
/// Constructed by the export pipeline based on the Layout configuration.
/// Provided to trait methods so they can compute cross-file references.
pub struct PathResolver {
    file_extension: String,
    index_file_stem: Option<String>,
    /// Precomputed map: module_path -> relative filesystem path (from export root)
    module_paths: HashMap<String, PathBuf>,
}

impl PathResolver {
    /// Build a `PathResolver` for the given types, constants, and layout configuration.
    pub fn new(
        file_extension: &str,
        index_file_stem: Option<&str>,
        layout: &Layout,
        types: &Types,
        constants: &Constants,
    ) -> Self {
        let mut module_paths = HashMap::new();

        if let Layout::MultiFile(config) = layout {
            let graph = build_module_graph(types, constants);
            collect_module_paths(
                &graph,
                config,
                file_extension,
                index_file_stem,
                &mut module_paths,
            );
        }

        PathResolver {
            file_extension: file_extension.to_string(),
            index_file_stem: index_file_stem.map(|s| s.to_string()),
            module_paths,
        }
    }

    /// Get the filesystem path (relative to export root) for a module.
    /// E.g., `"shared::item"` -> `"shared/item.ts"`
    pub fn module_file_path(&self, module_path: &str) -> Option<&Path> {
        self.module_paths.get(module_path).map(|p| p.as_path())
    }

    /// Get the filesystem path for a specific named type.
    /// E.g., `("User", "shared::item")` -> `"shared/item.ts"` (for TS)
    /// or `("User", "shared")` -> `"shared/User.schema.json"` (for JSON Schema)
    ///
    /// For languages where multiple types share a file (TS, Zod), this returns
    /// the module file path. For languages with one file per type (JSON Schema),
    /// this returns the type-specific path.
    pub fn type_file_path(&self, type_name: &str, module_path: &str) -> Option<PathBuf> {
        if let Some(module_file) = self.module_file_path(module_path) {
            // For most languages, types share a module file
            Some(module_file.to_path_buf())
        } else {
            // For per-type file layouts (e.g., JSON Schema), build the path
            let dir = if module_path.is_empty() {
                PathBuf::new()
            } else {
                PathBuf::from(module_path.replace("::", "/"))
            };
            let mut path = dir.join(type_name);
            path.set_extension(&self.file_extension);
            Some(path)
        }
    }

    /// Compute the relative import path from one module to another,
    /// using actual file paths when available (respects FolderGrouping).
    /// Falls back to module-path-based computation if files are not mapped.
    pub fn relative_import_path(&self, from_module: &str, to_module: &str) -> String {
        // Try using actual file paths first (handles FolderGrouping correctly)
        if let (Some(from_file), Some(to_file)) = (
            self.module_file_path(from_module).or_else(|| {
                (from_module.is_empty()).then(|| {
                    // Root module maps to the index file
                    std::path::Path::new("index")
                })
            }),
            self.module_file_path(to_module),
        ) {
            return relative_file_import_path(from_file, to_file, self.index_file_stem.as_deref());
        }
        // Fallback to segment-based computation
        relative_import_path(from_module, to_module)
    }

    /// Get the directory path for a module (the folder containing its file).
    pub fn module_dir_path(&self, module_path: &str) -> Option<PathBuf> {
        self.module_file_path(module_path)
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    }

    /// Get the index file path for a directory, if index files are enabled.
    /// E.g., for `"shared"` -> `"shared/index.ts"`
    pub fn index_file_path(&self, dir_module_path: &str) -> Option<PathBuf> {
        let stem = self.index_file_stem.as_deref()?;
        let dir = if dir_module_path.is_empty() {
            PathBuf::new()
        } else {
            PathBuf::from(dir_module_path.replace("::", "/"))
        };
        let mut path = dir.join(stem);
        path.set_extension(&self.file_extension);
        Some(path)
    }

    /// Get the file extension (without dot).
    pub fn file_extension(&self) -> &str {
        &self.file_extension
    }
}

/// Compute a relative import path between two module paths.
///
/// This is the core path resolution logic shared by all TypeScript-family exporters.
pub fn relative_import_path(from_module_path: &str, to_module_path: &str) -> String {
    fn module_file_segments(module_path: &str) -> Vec<&str> {
        if module_path.is_empty() {
            vec!["index"]
        } else {
            module_path.split("::").collect()
        }
    }

    let from_file_segments = module_file_segments(from_module_path);
    let from_dir_segments = &from_file_segments[..from_file_segments.len() - 1];
    let to_file_segments = module_file_segments(to_module_path);

    let shared = from_dir_segments
        .iter()
        .zip(to_file_segments.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let mut relative_parts = Vec::new();
    relative_parts.extend(std::iter::repeat_n(
        "..",
        from_dir_segments.len().saturating_sub(shared),
    ));
    relative_parts.extend(to_file_segments.iter().skip(shared).copied());

    if relative_parts
        .first()
        .is_none_or(|v| *v != "." && *v != "..")
    {
        relative_parts.insert(0, ".");
    }

    relative_parts.join("/")
}

/// Compute the module alias for namespace imports.
/// E.g., `"shared::item"` -> `"shared$item"`
pub fn module_alias(module_path: &str) -> String {
    if module_path.is_empty() {
        "$root".to_string()
    } else {
        module_path.split("::").collect::<Vec<_>>().join("$")
    }
}

/// Compute a relative import path between two actual file paths (without extensions).
/// E.g., from `a/b.ts` to `a/c_d.ts` → `"./c_d"`
/// E.g., from `a/b.ts` to `x/y.ts` → `"../x/y"`
///
/// When `index_file_stem` is provided and the target is an index file,
/// the stem is stripped to produce directory imports (e.g., `"./ex_shared"`
/// instead of `"./ex_shared/index"`), since TypeScript resolves directory
/// imports to the index file automatically.
fn relative_file_import_path(
    from_file: &Path,
    to_file: &Path,
    index_file_stem: Option<&str>,
) -> String {
    let from_dir_components: Vec<_> = from_file
        .parent()
        .map(|p| p.components().collect())
        .unwrap_or_default();
    let to_dir_components: Vec<_> = to_file
        .parent()
        .map(|p| p.components().collect())
        .unwrap_or_default();
    let to_stem = to_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("index");

    // Check if the target is an index file that can use directory import syntax
    let is_index = index_file_stem.is_some_and(|stem| to_stem == stem);

    let shared = from_dir_components
        .iter()
        .zip(to_dir_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let mut parts: Vec<&str> = Vec::new();
    // Go up from `from_dir` to the common ancestor
    for _ in 0..(from_dir_components.len() - shared) {
        parts.push("..");
    }
    // Go down to `to_dir`
    for c in &to_dir_components[shared..] {
        if let Some(s) = c.as_os_str().to_str() {
            parts.push(s);
        }
    }

    // For index files, use directory import (e.g., "./ex_shared" not "./ex_shared/index")
    // unless the result would be empty (same directory), in which case keep explicit stem
    if !is_index || parts.is_empty() {
        parts.push(to_stem);
    }

    // Ensure the path starts with `.` or `..`
    if parts.first().is_none_or(|v| *v != "." && *v != "..") {
        parts.insert(0, ".");
    }

    parts.join("/")
}

fn collect_module_paths(
    module: &Module<'_>,
    config: &MultiFileConfig,
    file_extension: &str,
    index_file_stem: Option<&str>,
    paths: &mut HashMap<String, PathBuf>,
) {
    // Root module maps to the index file
    if !module.module_path.is_empty() {
        let has_children = !module.children.is_empty();

        let path = match &config.folder_grouping {
            FolderGrouping::None => {
                let base = PathBuf::from(module.module_path.replace("::", "/"));
                if has_children {
                    if let Some(stem) = index_file_stem {
                        // Module with children: place inside module dir as index file
                        // e.g., ex_shared -> ex_shared/index.ts
                        let mut p = base.join(stem);
                        p.set_extension(file_extension);
                        p
                    } else {
                        let mut p = base;
                        p.set_extension(file_extension);
                        p
                    }
                } else {
                    let mut p = base;
                    p.set_extension(file_extension);
                    p
                }
            }
            FolderGrouping::ByDepth(depth) => {
                let segments: Vec<&str> = module.module_path.split("::").collect();
                let (folder_segments, file_segments) = if segments.len() <= *depth {
                    (
                        &segments[..segments.len().saturating_sub(1)],
                        &segments[segments.len().saturating_sub(1)..],
                    )
                } else {
                    (&segments[..*depth], &segments[*depth..])
                };

                let folder = folder_segments.join("/");
                let file_name = file_segments.join("_");

                // Check if this module has children that would create a
                // subdirectory conflicting with this module's file name.
                // This happens when children go into a folder named after
                // this module's file_name.
                let needs_index = has_children && index_file_stem.is_some() && {
                    // A conflict exists when the computed file_name could also
                    // be a directory for child modules. This happens when any
                    // child's path would start with this module's directory.
                    // Simplest check: if children exist and we have index support,
                    // always use index file to avoid potential conflicts.
                    true
                };

                if needs_index {
                    let dir = if folder.is_empty() {
                        PathBuf::from(&file_name)
                    } else {
                        PathBuf::from(&folder).join(&file_name)
                    };
                    let mut p = dir.join(index_file_stem.unwrap());
                    p.set_extension(file_extension);
                    p
                } else {
                    let mut p = if folder.is_empty() {
                        PathBuf::from(&file_name)
                    } else {
                        PathBuf::from(&folder).join(&file_name)
                    };
                    p.set_extension(file_extension);
                    p
                }
            }
        };
        paths.insert(module.module_path.to_string(), path);
    }

    for (_, child) in &module.children {
        collect_module_paths(child, config, file_extension, index_file_stem, paths);
    }
}
