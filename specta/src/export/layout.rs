use std::fmt;

/// Controls how generated types are organized on disk.
/// Shared across all exporter crates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layout {
    /// All types in a single file.
    SingleFile(SingleFileConfig),

    /// One file per Rust module path. Cross-module references use
    /// language-specific import/reference mechanisms.
    MultiFile(MultiFileConfig),
}

impl Default for Layout {
    fn default() -> Self {
        Layout::SingleFile(SingleFileConfig::default())
    }
}

impl Layout {
    /// Shorthand for `Layout::MultiFile(MultiFileConfig::default())`.
    pub fn multi_file() -> Self {
        Layout::MultiFile(MultiFileConfig::default())
    }

    /// Shorthand for `Layout::SingleFile(SingleFileConfig { module_prefix_names: true })`.
    pub fn module_prefixed() -> Self {
        Layout::SingleFile(SingleFileConfig {
            module_prefix_names: true,
        })
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Layout::SingleFile(config) => {
                if config.module_prefix_names {
                    write!(f, "SingleFile(ModulePrefixed)")
                } else {
                    write!(f, "SingleFile")
                }
            }
            Layout::MultiFile(_) => write!(f, "MultiFile"),
        }
    }
}

/// Configuration for `Layout::SingleFile`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SingleFileConfig {
    /// Whether to prefix type names with their module path
    /// (e.g., `shared_item_User` instead of just `User`).
    pub module_prefix_names: bool,
}

/// Configuration for `Layout::MultiFile`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiFileConfig {
    /// How to group modules into folders.
    pub folder_grouping: FolderGrouping,

    /// Whether and how to generate index/barrel files.
    pub index_files: IndexFileConfig,

    /// How cross-module imports are written (language-specific behavior
    /// is in the trait, but this controls the structural style).
    pub import_style: ImportStyle,
}

impl Default for MultiFileConfig {
    fn default() -> Self {
        Self {
            folder_grouping: FolderGrouping::None,
            index_files: IndexFileConfig::None,
            import_style: ImportStyle::Namespace,
        }
    }
}

/// Controls how module paths are grouped into filesystem folders.
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

/// Controls whether and how index/barrel files are generated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum IndexFileConfig {
    /// Do not generate index/barrel files.
    #[default]
    None,

    /// Generate an index file in each folder that re-exports
    /// everything from that folder's files.
    ReExportAll,
}

/// Controls how cross-module imports are structured.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ImportStyle {
    /// Namespace/wildcard imports: `import * as alias from "./path";`
    #[default]
    Namespace,

    /// Named imports: `import { TypeA, TypeB } from "./path";`
    /// References use the type name directly instead of `alias.TypeName`.
    Named,
}
