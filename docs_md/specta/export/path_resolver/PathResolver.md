# Struct `PathResolver` -- path: `specta::export::path_resolver::PathResolver`

Resolves module paths and named types to filesystem paths.

Constructed by the export pipeline based on the Layout configuration.
Provided to trait methods so they can compute cross-file references.

```rust
pub struct PathResolver {
	file_extension: String,
	index_file_stem: Option<String>,
	/// Precomputed map: module_path -> relative filesystem path (from export root)
	module_paths: std::collections::HashMap<String, std::path::PathBuf>,
}
```

## Methods

```rust
/**
`new` -- Build a `PathResolver` for the given types, constants, and layout configuration.
*/
pub fn new(file_extension: &str, index_file_stem: Option<&str>, layout: &Layout, types: &Types, constants: &Constants) -> Self
/**
`module_file_path` -- Get the filesystem path (relative to export root) for a module.
E.g., `"shared::item"` -> `"shared/item.ts"`
*/
pub fn module_file_path(self: &Self, module_path: &str) -> Option<&Path>
/**
`type_file_path` -- Get the filesystem path for a specific named type.
E.g., `("User", "shared::item")` -> `"shared/item.ts"` (for TS)
or `("User", "shared")` -> `"shared/User.schema.json"` (for JSON Schema)

For languages where multiple types share a file (TS, Zod), this returns
the module file path. For languages with one file per type (JSON Schema),
this returns the type-specific path.
*/
pub fn type_file_path(self: &Self, type_name: &str, module_path: &str) -> Option<PathBuf>
/**
`relative_import_path` -- Compute the relative import path from one module to another,
using actual file paths when available (respects FolderGrouping).
Falls back to module-path-based computation if files are not mapped.
*/
pub fn relative_import_path(self: &Self, from_module: &str, to_module: &str) -> String
/**
`module_dir_path` -- Get the directory path for a module (the folder containing its file).
*/
pub fn module_dir_path(self: &Self, module_path: &str) -> Option<PathBuf>
/**
`index_file_path` -- Get the index file path for a directory, if index files are enabled.
E.g., for `"shared"` -> `"shared/index.ts"`
*/
pub fn index_file_path(self: &Self, dir_module_path: &str) -> Option<PathBuf>
/**
`file_extension` -- Get the file extension (without dot).
*/
pub fn file_extension(self: &Self) -> &str
```

