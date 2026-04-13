---
id: TASK-004
title: Unify export path and file creation
status: To Do
assignee: []
created_date: '2026-04-13 07:03'
labels: []
dependencies: []
---

# Unify Export Path and File Creation

## Problem

The file export pipeline — module graph construction, filesystem path resolution, import generation, stale file cleanup — is duplicated nearly line-for-line across `specta-typescript`, `specta-zod`, and partially in `specta-jsonschema`. Both TypeScript and Zod output `.ts` files and share the exact same module resolution and import semantics, yet each crate re-implements everything independently. This makes bugs and feature work (like new layout modes) require parallel changes in multiple places.

### Current Duplicated Functions

| Function | TS (`exporter.rs`) | Zod (`zod.rs`) | Identical? |
|---|---|---|---|
| `module_alias()` | L954-960 | L681-687 | Exact |
| `module_import_path()` | L1011-1045 | L705-739 | Exact |
| `module_import_statement()` | L962-980 | L689-695 | Similar (TS has `import type` logic) |
| `module_import_block()` | L982-1009 | L697-703 | Similar (TS has JSDoc variant) |
| `render_file_header()` | L690-702 | L510-522 | Exact (minus return type) |
| `exported_type_name()` | L942-952 | L667-679 | Near-identical |
| `render_flat_types()` | L817-845 | L544-573 | Near-identical (Zod adds topo sort) |
| `build_module_graph()` | L627-688 | L474-508 | Identical core (TS adds constants) |
| `Module` struct | L620-625 | L468-472 | Identical (TS adds `constants` field) |
| `collect_existing_files()` | L847-875 | L575-602 | Exact (TS: `.ts|.js`, Zod: `.ts`) |
| `is_generated_specta_file()` | L877-883 | L604-610 | Exact |
| `remove_empty_dirs()` | L886-919 | L612-645 | Exact |
| `cleanup_stale_files()` | L922-939 | L647-665 | Exact |
| `export_to()` inner `export()` | L263-359 | L216-286 | Same structure, TS has constants + `import type` |

Zod also has `topological_sort_types()` (L742-810) that TypeScript does not, but will benefit from sharing.

JSON Schema (`specta-jsonschema/src/json_schema.rs`) has its own `Layout` enum (`SingleFile`, `Files`) and separate `export_files()` logic that independently resolves module paths to filesystem directories and builds `$ref` URIs. It doesn't share any code with the TS/Zod exporters despite solving the same structural problem: "given a named type, what file does it live in and how do I reference it?"

---

## Goal

Create a **language-agnostic** export pipeline in the `specta` core crate that all exporter crates use. Individual exporters implement a trait to provide language-specific behavior (file extension, import syntax, type rendering). The pipeline handles everything structural: module graph construction, filesystem layout, path resolution, file writing, stale file cleanup, and index/barrel file generation.

### New capabilities:

1. **Module-prefix folder grouping** — types with module path `shared::item` and `shared::item::data` go into a `shared/` folder; types with `tools::declarations` go into a `tools/` folder. Grouping depth is configurable.
2. **Index/barrel file generation** — configurable per-language (e.g., `index.ts` for TypeScript, `mod.rs` for Rust, `__init__.py` for Python). Extension and naming come from the trait, not hardcoded.
3. **Named imports** — support `import { Name } from "..."` in addition to `import * as alias from "..."` for TypeScript-family exporters.
4. **Type path resolution** — given any `NamedDataType`, resolve the filesystem path where it will be exported. This lets JSON Schema compute `$ref` URIs, TypeScript compute import paths, and any future language exporter locate types.

### Unification of Zod and TypeScript

Since both Zod and TypeScript output `.ts` files, they must share the same module resolution, file/import creation, and index file generation. The only difference is what each type's body looks like (TS type alias vs Zod schema `const`). Both exporters implement the same trait and delegate all structural work to the shared pipeline.

---

## Architecture

### Location: `specta/src/export/` module

All shared export infrastructure lives in the `specta` core crate under a new `export` module. This is the right location because:

- `specta` is already depended on by every exporter crate
- The pipeline operates on `Types`, `Constants`, `NamedDataType` — all core types
- No new crate is needed; no circular dependencies are introduced
- Language-specific crates only need to implement a trait, not depend on each other

### Module structure

```
specta/src/export/
├── mod.rs              — public API, re-exports, ExportLanguage trait
├── module_graph.rs     — Module struct, build_module_graph()
├── layout.rs           — Layout enum, FilesConfig, FolderGrouping, IndexFileConfig, ImportStyle
├── path_resolver.rs    — resolve NamedDataType -> filesystem path, reference path
├── pipeline.rs         — ExportPipeline: the main orchestrator for file export
├── filesystem.rs       — file writing, stale cleanup, directory management
├── topo_sort.rs        — topological_sort_types(), collect_type_deps(), collect_fields_deps()
```

### The `ExportLanguage` trait

This is the core abstraction. Each exporter crate implements it once.

```rust
/// Trait implemented by language exporters to plug into the shared export pipeline.
///
/// The pipeline handles structural concerns (module graph, file layout, path resolution,
/// stale cleanup). The trait provides language-specific behavior (rendering, import syntax,
/// file extensions).
pub trait ExportLanguage {
    /// The exporter's error type.
    type Error: From<std::io::Error>;

    /// File extension for generated files, without the dot (e.g., `"ts"`, `"json"`, `"swift"`).
    fn file_extension(&self) -> &str;

    /// Extensions to match when scanning for stale generated files.
    /// Defaults to `[self.file_extension()]`. Override to include multiple
    /// (e.g., `["ts", "js"]` for TypeScript with JSDoc).
    fn stale_file_extensions(&self) -> Vec<&str> {
        vec![self.file_extension()]
    }

    /// Name of the index/barrel file in each directory (without extension).
    /// Return `None` to disable index file generation entirely.
    /// Examples: `Some("index")` for TS, `Some("mod")` for Rust, `None` for JSON Schema.
    fn index_file_stem(&self) -> Option<&str>;

    /// The string marker used to identify generated files for stale cleanup.
    /// Defaults to `"generated by Specta"`.
    fn generated_marker(&self) -> &str {
        "generated by Specta"
    }

    /// Render the file header (comments, preludes, framework imports).
    /// Called once per generated file.
    fn render_file_header(&self) -> String;

    /// Render the body for a single module's types.
    ///
    /// `types` is the full type collection (for reference lookups).
    /// `module_types` are the types belonging to this specific module, pre-sorted.
    /// `indent` is the indentation prefix (used for namespace nesting).
    ///
    /// Returns the rendered string and a set of referenced module paths
    /// that this module's types depend on.
    fn render_module_types(
        &self,
        types: &Types,
        module_types: &[&NamedDataType],
        indent: &str,
    ) -> Result<ModuleRenderResult, Self::Error>;

    /// Render import/reference statements for cross-module dependencies.
    ///
    /// `from_module_path` is the Rust module path of the file being rendered.
    /// `imports` maps target module paths to the set of type names imported from each.
    /// `path_resolver` can resolve any module path to its relative filesystem path.
    ///
    /// Return the import block as a string (may be empty if the language doesn't
    /// use imports, e.g., JSON Schema uses `$ref` inline instead).
    fn render_imports(
        &self,
        from_module_path: &str,
        imports: &BTreeMap<String, ImportInfo>,
        path_resolver: &PathResolver,
    ) -> Result<String, Self::Error>;

    /// Render the contents of an index/barrel file for a directory.
    ///
    /// `files` lists the (file_stem, exported_names) pairs for each file in the directory.
    /// `subdirectories` lists subdirectory names that have their own index files.
    /// `path_resolver` resolves relative paths.
    ///
    /// Only called if `index_file_stem()` returns `Some(...)`.
    fn render_index_file(
        &self,
        files: &[IndexFileEntry],
        subdirectories: &[&str],
        path_resolver: &PathResolver,
    ) -> Result<String, Self::Error> {
        let _ = (files, subdirectories, path_resolver);
        Ok(String::new())
    }

    /// Render constants belonging to a module. Default: no-op.
    fn render_constants(
        &self,
        _constants: &[&NamedConstant],
    ) -> Result<String, Self::Error> {
        Ok(String::new())
    }

    /// Compute the exported name for a type given the current layout.
    /// Default: uses module-prefixed name for `ModulePrefixedName` layout,
    /// bare name otherwise.
    fn exported_type_name(&self, layout: &Layout, ndt: &NamedDataType) -> Cow<'static, str> {
        match layout {
            Layout::ModulePrefixedName => {
                let mut s = ndt.module_path().split("::").collect::<Vec<_>>().join("_");
                if !s.is_empty() {
                    s.push('_');
                }
                s.push_str(ndt.name());
                Cow::Owned(s)
            }
            _ => ndt.name().clone(),
        }
    }
}
```

### Supporting types for the trait

```rust
/// Result of rendering a module's types.
pub struct ModuleRenderResult {
    /// The rendered type declarations as a string.
    pub body: String,
    /// Map of exported type names to their source locations (for duplicate detection).
    pub exports: HashMap<String, Location<'static>>,
    /// Set of module paths that this module's types reference.
    /// The pipeline uses this to generate import statements.
    pub referenced_modules: BTreeMap<String, ReferencedImport>,
}

/// Information about what is imported from a specific module.
pub struct ImportInfo {
    /// The specific type names imported from this module.
    pub type_names: BTreeSet<String>,
    /// Whether any imported item is a runtime value (not just a type).
    /// Relevant for TypeScript `import` vs `import type` distinction.
    pub has_values: bool,
}

/// An entry in an index/barrel file.
pub struct IndexFileEntry<'a> {
    /// File stem (without extension), e.g., `"item"` for `item.ts`.
    pub file_stem: &'a str,
    /// Names exported by this file.
    pub exported_names: &'a [String],
    /// The module path this file corresponds to.
    pub module_path: &'a str,
}
```

### PathResolver

The `PathResolver` is constructed by the pipeline and made available to trait methods. It resolves any named type or module path to its filesystem path relative to the export root. This is essential for:

- **TypeScript/Zod**: computing relative import paths between files
- **JSON Schema**: computing `$ref` URIs (e.g., `./shared/User.schema.json`)
- **Any language**: locating where a type will be written on disk

```rust
/// Resolves module paths and named types to filesystem paths.
///
/// Constructed by the export pipeline based on the Layout configuration.
/// Provided to trait methods so they can compute cross-file references.
pub struct PathResolver<'a> {
    layout: &'a Layout,
    file_extension: &'a str,
    index_file_stem: Option<&'a str>,
    /// Precomputed map: module_path -> relative filesystem path (from export root)
    module_paths: HashMap<String, PathBuf>,
    /// Precomputed map: (type_name, module_path) -> relative filesystem path
    type_paths: HashMap<(String, String), PathBuf>,
}

impl PathResolver<'_> {
    /// Get the filesystem path (relative to export root) for a module.
    /// E.g., `"shared::item"` -> `"shared/item.ts"`
    pub fn module_file_path(&self, module_path: &str) -> Option<&Path>;

    /// Get the filesystem path for a specific named type.
    /// E.g., `("User", "shared::item")` -> `"shared/item.ts"` (for TS)
    /// or `("User", "shared")` -> `"shared/User.schema.json"` (for JSON Schema)
    pub fn type_file_path(&self, type_name: &str, module_path: &str) -> Option<&Path>;

    /// Compute the relative import path from one module to another.
    /// E.g., from `"ex_app"` to `"ex_shared"` -> `"./ex_shared"`
    /// Handles `..` traversal for nested modules.
    pub fn relative_import_path(&self, from_module: &str, to_module: &str) -> String;

    /// Get the directory path for a module (the folder containing its file).
    pub fn module_dir_path(&self, module_path: &str) -> Option<PathBuf>;

    /// Get the index file path for a directory, if index files are enabled.
    /// E.g., for `"shared"` -> `"shared/index.ts"`
    pub fn index_file_path(&self, dir_module_path: &str) -> Option<PathBuf>;
}
```

### Layout enum (unified, in `specta`)

```rust
/// Controls how generated types are organized on disk.
/// Shared across all exporter crates.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Layout {
    /// All types in a single file, no module structure.
    #[default]
    FlatFile,

    /// All types in a single file, with module path prefixed to each name
    /// (e.g., `shared_item_User`).
    ModulePrefixedName,

    /// One file per Rust module path. Cross-module references use
    /// language-specific import/reference mechanisms.
    Files(FilesConfig),
}

impl Layout {
    /// Shorthand for `Layout::Files(FilesConfig::default())`.
    pub fn files() -> Self {
        Layout::Files(FilesConfig::default())
    }
}

/// Configuration for `Layout::Files`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesConfig {
    /// How to group modules into folders.
    pub folder_grouping: FolderGrouping,

    /// Whether and how to generate index/barrel files.
    pub index_files: IndexFileConfig,

    /// How cross-module imports are written (language-specific behavior
    /// is in the trait, but this controls the structural style).
    pub import_style: ImportStyle,
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            folder_grouping: FolderGrouping::None,
            index_files: IndexFileConfig::None,
            import_style: ImportStyle::Namespace,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum FolderGrouping {
    /// No folder grouping; current behavior. Each module path segment
    /// maps to a nested file. `shared::item` -> `shared/item.{ext}`
    /// with `shared.{ext}` for types directly in `shared`.
    #[default]
    None,

    /// Group by the first N segments of the module path into folders.
    /// With depth=1: `shared::item` and `shared::item::data` both go
    /// into the `shared/` folder. `tools::declarations` goes into `tools/`.
    ByDepth(usize),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum IndexFileConfig {
    /// Do not generate index/barrel files.
    #[default]
    None,

    /// Generate an index file in each folder that re-exports
    /// everything from that folder's files.
    ReExportAll {
        /// If true, re-exports use the module path prefix:
        /// `export { User as shared_User } from "./item";`
        /// If false, re-exports are flat:
        /// `export { User } from "./item";`
        use_module_prefix: bool,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ImportStyle {
    /// Namespace/wildcard imports: `import * as alias from "./path";`
    #[default]
    Namespace,

    /// Named imports: `import { TypeA, TypeB } from "./path";`
    /// References use the type name directly instead of `alias.TypeName`.
    Named,
}
```

### Module graph (in `specta`)

```rust
/// A node in the module tree. Represents a Rust module that contains
/// types, constants, and child modules.
pub struct Module<'a> {
    /// Types belonging directly to this module.
    pub types: Vec<&'a NamedDataType>,
    /// Constants belonging directly to this module.
    pub constants: Vec<&'a NamedConstant>,
    /// Child modules keyed by segment name.
    pub children: BTreeMap<&'a str, Module<'a>>,
    /// Full Rust module path (e.g., `"shared::item"`).
    pub module_path: Cow<'static, str>,
}

/// Build a module tree from a flat collection of types and constants.
pub fn build_module_graph<'a>(types: &'a Types, constants: &'a Constants) -> Module<'a>;
```

### ExportPipeline (in `specta`)

```rust
/// The main orchestrator for file-based export.
///
/// Handles module graph construction, filesystem layout, path resolution,
/// file writing, and stale file cleanup. Delegates language-specific
/// rendering to the `ExportLanguage` trait.
pub struct ExportPipeline;

impl ExportPipeline {
    /// Export types to a directory using the given language exporter and layout.
    ///
    /// For `Layout::FlatFile` and `Layout::ModulePrefixedName`, `path` is a file path.
    /// For `Layout::Files`, `path` is a directory.
    pub fn export_to<L: ExportLanguage>(
        lang: &L,
        layout: &Layout,
        path: &Path,
        resolved: &ResolvedTypes,
    ) -> Result<(), L::Error>;

    /// Export types to a single string (for `FlatFile` and `ModulePrefixedName` layouts).
    pub fn export_to_string<L: ExportLanguage>(
        lang: &L,
        layout: &Layout,
        resolved: &ResolvedTypes,
    ) -> Result<String, L::Error>;

    /// Build a PathResolver for the given types and layout without writing files.
    /// Useful for computing $ref URIs (JSON Schema) or import paths ahead of time.
    pub fn build_path_resolver<L: ExportLanguage>(
        lang: &L,
        layout: &Layout,
        types: &Types,
        constants: &Constants,
    ) -> PathResolver;
}
```

---

## Example: how each exporter implements the trait

### TypeScript

```rust
impl ExportLanguage for Exporter {
    type Error = Error;

    fn file_extension(&self) -> &str {
        if self.jsdoc { "js" } else { "ts" }
    }

    fn stale_file_extensions(&self) -> Vec<&str> {
        vec!["ts", "js"]
    }

    fn index_file_stem(&self) -> Option<&str> {
        Some("index")
    }

    fn render_file_header(&self) -> String {
        let mut out = self.header.to_string();
        if !self.header.is_empty() { out.push('\n'); }
        out.push_str(&self.framework_prelude);
        if !self.framework_prelude.is_empty() { out.push('\n'); }
        out
    }

    fn render_module_types(&self, types: &Types, module_types: &[&NamedDataType], indent: &str)
        -> Result<ModuleRenderResult, Error>
    {
        // Calls existing primitives::export_internal()
        // Collects references via references::collect_references()
        // Returns body + exports + referenced_modules
    }

    fn render_imports(&self, from: &str, imports: &BTreeMap<String, ImportInfo>, resolver: &PathResolver)
        -> Result<String, Error>
    {
        // Uses resolver.relative_import_path() for paths
        // Chooses `import type` vs `import` based on ImportInfo::has_values
        // Handles JSDoc @typedef syntax when self.jsdoc is true
        // Handles ImportStyle::Named vs Namespace from the config
    }

    fn render_index_file(&self, files: &[IndexFileEntry], subdirs: &[&str], resolver: &PathResolver)
        -> Result<String, Error>
    {
        // Generates `export { ... } from "./file";` for each entry
        // Generates `export * from "./subdir";` for each subdirectory
    }

    fn render_constants(&self, constants: &[&NamedConstant]) -> Result<String, Error> {
        // Calls existing constants::export_constant_internal()
    }
}
```

### Zod

```rust
impl ExportLanguage for Zod {
    type Error = Error;

    fn file_extension(&self) -> &str { "ts" }

    fn index_file_stem(&self) -> Option<&str> { Some("index") }

    fn render_file_header(&self) -> String {
        // Same pattern as TS — header + prelude (includes `import { z } from "zod"`)
    }

    fn render_module_types(&self, types: &Types, module_types: &[&NamedDataType], indent: &str)
        -> Result<ModuleRenderResult, Error>
    {
        // Calls zod primitives::export_internal()
        // Uses topological sorting (now from specta::export::topo_sort)
    }

    fn render_imports(&self, from: &str, imports: &BTreeMap<String, ImportInfo>, resolver: &PathResolver)
        -> Result<String, Error>
    {
        // Same as TS but always `import` (never `import type`) since Zod schemas are runtime values
        // Shares the same structural logic because both are TypeScript output
    }

    fn render_index_file(&self, files: &[IndexFileEntry], subdirs: &[&str], resolver: &PathResolver)
        -> Result<String, Error>
    {
        // Same as TS — both produce .ts barrel files
    }
}
```

### JSON Schema

```rust
impl ExportLanguage for JsonSchema {
    type Error = Error;

    fn file_extension(&self) -> &str { "schema.json" }

    fn index_file_stem(&self) -> Option<&str> { None }  // No barrel files

    fn render_file_header(&self) -> String { String::new() }  // JSON has no header

    fn render_module_types(&self, types: &Types, module_types: &[&NamedDataType], indent: &str)
        -> Result<ModuleRenderResult, Error>
    {
        // For Files layout: each type gets its own file, so this renders one schema per type
        // Uses resolver to compute $ref URIs instead of hardcoding path logic
    }

    fn render_imports(&self, _from: &str, _imports: &BTreeMap<String, ImportInfo>, _resolver: &PathResolver)
        -> Result<String, Error>
    {
        Ok(String::new())  // JSON Schema uses inline $ref, not import statements
    }
}
```

The key insight for JSON Schema: it doesn't use `render_imports` because `$ref` URIs are computed inline during `render_module_types` using the `PathResolver`. The pipeline still builds the path resolver and provides it — JSON Schema just uses it at a different point in the process.

---

## Example: desired output for folder grouping with index files

Given module paths: `shared::item`, `shared::item::data`, `tools::declarations`

With `Layout::Files(FilesConfig { folder_grouping: ByDepth(1), index_files: ReExportAll { use_module_prefix: false }, .. })`:

```
output/
├── shared/
│   ├── item.ts              # types from shared::item
│   ├── item_data.ts         # types from shared::item::data
│   └── index.ts             # export { User, ... } from "./item";
│                             # export { DataPayload, ... } from "./item_data";
├── tools/
│   ├── declarations.ts      # types from tools::declarations
│   └── index.ts             # export { MyDecl, ... } from "./declarations";
└── index.ts                 # root types + re-exports from folders
```

With `ImportStyle::Named`, cross-module references become:
```typescript
import { User, Role } from "./shared/item";
// usage: field: User  (not shared$item.User)
```

Instead of current:
```typescript
import * as shared$item from "./shared/item";
// usage: field: shared$item.User
```

### JSON Schema with the same module structure:

```
output/
├── shared/
│   ├── User.schema.json
│   ├── Pagination.schema.json
│   └── ...
├── tools/
│   └── MyDecl.schema.json
```

JSON Schema uses `PathResolver::type_file_path("User", "shared::item")` -> `"shared/User.schema.json"` to compute `$ref` URIs. The same resolver that TypeScript uses for import paths.

---

## Implementation Plan

### Phase 1: Create `specta/src/export/` module with core types

Create the module structure and define:
- `Layout` enum, `FilesConfig`, `FolderGrouping`, `IndexFileConfig`, `ImportStyle`
- `Module` struct, `build_module_graph()`
- `ExportLanguage` trait (initially with just the method signatures)
- `PathResolver` struct with resolution methods
- `ModuleRenderResult`, `ImportInfo`, `IndexFileEntry`

Re-export from `specta/src/lib.rs` as `pub mod export`.

**Files to create:**
- `specta/src/export/mod.rs`
- `specta/src/export/layout.rs`
- `specta/src/export/module_graph.rs`
- `specta/src/export/path_resolver.rs`

**Files to modify:**
- `specta/src/lib.rs` — add `pub mod export`

### Phase 2: Extract filesystem utilities into `specta`

Move the following functions (currently duplicated in TS and Zod) into `specta/src/export/filesystem.rs`, parameterized by extensions and generated marker:

- `collect_existing_files(root, extensions: &[&str])` — parameterize file extension filter
- `is_generated_file(path, marker: &str)` — parameterize the marker string
- `remove_empty_dirs(path, root)` — already generic
- `cleanup_stale_files(root, current_files, extensions, marker)` — parameterize

**Files to create:**
- `specta/src/export/filesystem.rs`

**Files to modify (later, once pipeline is ready):**
- `specta-typescript/src/exporter.rs` — remove local copies
- `specta-zod/src/zod.rs` — remove local copies

### Phase 3: Extract topological sort into `specta`

Move from `specta-zod/src/zod.rs` L742-897 into `specta/src/export/topo_sort.rs`:

- `topological_sort_types()`
- `collect_type_deps()`
- `collect_fields_deps()`

These operate purely on `DataType`, `Types`, and `NamedDataType` — all core types. No exporter-specific logic.

**Files to create:**
- `specta/src/export/topo_sort.rs`

### Phase 4: Build the `ExportPipeline`

Create `specta/src/export/pipeline.rs` with the full file export orchestration:

1. Build module graph from `Types` + `Constants`
2. Construct `PathResolver` from `Layout` + `ExportLanguage` config
3. For each module in the graph:
   a. Call `lang.render_module_types()` to get body + references
   b. Call `lang.render_imports()` with resolved paths
   c. Compose file: header + imports + body + constants
   d. Add to file map with resolved filesystem path
4. If index files enabled, call `lang.render_index_file()` for each directory
5. Handle framework runtime (via an optional closure, preserving current pattern)
6. Write all files to disk
7. Call `cleanup_stale_files()` to remove orphaned generated files

The pipeline replaces the `export_to` inner `export()` function that is currently duplicated in both TS and Zod.

**Files to create:**
- `specta/src/export/pipeline.rs`

### Phase 5: Implement `ExportLanguage` for TypeScript

Implement the trait on `specta_typescript::Exporter`. This means:
- Moving rendering logic to trait methods
- Replacing the local `export_to` with a delegation to `ExportPipeline::export_to`
- Removing all duplicated structural functions (module_alias, module_import_path, etc.)
- Keeping only TS-specific logic: `primitives::export_internal`, `constants::export_constant_internal`, branded types, JSDoc variant

The `Typescript` and `JSDoc` wrapper structs continue to exist and delegate to `Exporter`, which now implements `ExportLanguage`.

**Files to modify:**
- `specta-typescript/src/exporter.rs` — major refactor
- `specta-typescript/src/lib.rs` — re-export `Layout` from `specta::export`
- `specta-typescript/Cargo.toml` — may need feature flag for `specta/export`

### Phase 6: Implement `ExportLanguage` for Zod

Implement the trait on `specta_zod::Zod`. Same pattern as TypeScript:
- Delegate `export_to` to `ExportPipeline`
- Remove all duplicated functions
- Keep only Zod-specific rendering (`primitives::export_internal` for Zod)

`specta_zod::Layout` becomes a re-export of `specta::export::Layout`.

**Files to modify:**
- `specta-zod/src/zod.rs` — major refactor
- `specta-zod/src/lib.rs` — re-export `Layout` from `specta::export`

### Phase 7: Implement `ExportLanguage` for JSON Schema

Implement the trait on `specta_jsonschema::JsonSchema`:
- `file_extension()` returns `"schema.json"`
- `index_file_stem()` returns `None`
- `render_imports()` returns empty string
- `render_module_types()` uses `PathResolver::type_file_path()` to compute `$ref` URIs

This replaces the hardcoded `build_ref()` method and the manual `export_files()` function with the shared pipeline.

`specta_jsonschema::Layout` (`SingleFile`, `Files`) maps to `specta::export::Layout` (`FlatFile`, `Files(default)`).

**Files to modify:**
- `specta-jsonschema/src/json_schema.rs` — refactor to use pipeline
- `specta-jsonschema/src/layout.rs` — re-export from `specta::export`
- `specta-jsonschema/src/lib.rs`

### Phase 8: Implement `FolderGrouping`

In `PathResolver` and `pipeline.rs`, add folder grouping logic:

- `FolderGrouping::None` — current behavior
- `FolderGrouping::ByDepth(n)` — the first `n` segments of the module path determine the folder. Remaining segments are joined with `_` to form the filename.

Example with `ByDepth(1)`:
- `shared::item` -> folder `shared/`, file `item.{ext}`
- `shared::item::data` -> folder `shared/`, file `item_data.{ext}`
- `tools::declarations` -> folder `tools/`, file `declarations.{ext}`

The `PathResolver` must account for this in `relative_import_path()` since the filesystem structure no longer mirrors `::` segments 1:1.

### Phase 9: Implement index file generation

After all module files are computed in the pipeline:

1. For each directory that contains generated files, collect the files and their exported names
2. Build `IndexFileEntry` structs
3. Call `lang.render_index_file()` with the entries
4. Write the index file (using the stem from `lang.index_file_stem()` + extension from `lang.file_extension()`)
5. Root index file also re-exports from subdirectories

The trait method `render_index_file` is language-specific:
- **TypeScript**: `export { A, B } from "./file";` or `export * from "./subdir";`
- **Zod**: same as TypeScript (both produce `.ts`)
- **JSON Schema**: returns `None` for `index_file_stem()`, so no index files

### Phase 10: Implement `ImportStyle::Named`

When `ImportStyle::Named` is active:

1. The pipeline passes per-type import info (not just module paths) to `render_imports`
2. `render_imports` generates `import { TypeA, TypeB } from "./path"` 
3. The reference rendering layer must output bare `TypeA` instead of `alias.TypeA`

**Collision handling**: If two modules export a type with the same name, use aliased named imports: `import { Foo as shared_Foo } from "./shared"`. The pipeline detects collisions when building import info and provides alias suggestions in `ImportInfo`.

This requires coordination between the pipeline (which knows about collisions) and the reference renderer (which emits type names). Add a method or callback for the exporter to query the resolved name of a referenced type.

---

## Migration Strategy

Each phase is independently shippable. After each phase:
1. Run `cargo test -p specta -p specta-typescript -p specta-zod -p specta-jsonschema`
2. Run `ex-app` export tests to verify output stability
3. Old functions can remain as thin wrappers during migration, then get removed

### Breaking Changes

- **`Layout` moves to `specta::export`**: exporter crates re-export it. Users using `specta_typescript::Layout` continue to work via re-export. Users pattern-matching on `Layout::Files` need to update to `Layout::Files(config)` — provide `Layout::files()` shorthand.
- **`specta_jsonschema::Layout`** variants rename: `SingleFile` -> `FlatFile` to match the unified enum. Provide a deprecated alias.
- New `ExportLanguage` trait is an internal implementation detail; end users call the same `exporter.export_to(path, types)` API they always have.

### Feature gating

The `export` module in `specta` should be behind a feature flag (e.g., `feature = "export"`) to avoid bloating the core crate for users who only need type collection. The `std` feature should enable it by default since file I/O requires `std`.

---

## Files Summary

### New files (in `specta` crate)
- `specta/src/export/mod.rs` — `ExportLanguage` trait, re-exports
- `specta/src/export/layout.rs` — `Layout`, `FilesConfig`, `FolderGrouping`, `IndexFileConfig`, `ImportStyle`
- `specta/src/export/module_graph.rs` — `Module`, `build_module_graph()`
- `specta/src/export/path_resolver.rs` — `PathResolver`
- `specta/src/export/pipeline.rs` — `ExportPipeline`
- `specta/src/export/filesystem.rs` — stale file cleanup, directory management
- `specta/src/export/topo_sort.rs` — topological sorting

### Modified files
- `specta/src/lib.rs` — add `pub mod export`
- `specta/Cargo.toml` — add `export` feature flag if needed
- `specta-typescript/src/exporter.rs` — implement `ExportLanguage`, remove duplicated functions, delegate `export_to` to pipeline
- `specta-typescript/src/lib.rs` — re-export `Layout` from `specta::export`
- `specta-zod/src/zod.rs` — implement `ExportLanguage`, remove duplicated functions, delegate `export_to` to pipeline
- `specta-zod/src/lib.rs` — re-export `Layout` from `specta::export`
- `specta-jsonschema/src/json_schema.rs` — implement `ExportLanguage`, use `PathResolver` for `$ref` URIs
- `specta-jsonschema/src/layout.rs` — re-export from `specta::export`
- `specta-jsonschema/src/lib.rs`
