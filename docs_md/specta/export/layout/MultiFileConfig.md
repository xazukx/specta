# Struct `MultiFileConfig` -- path: `specta::export::layout::MultiFileConfig`

Configuration for `Layout::MultiFile`.

```rust
pub struct MultiFileConfig {
	/// How to group modules into folders.
	pub folder_grouping: FolderGrouping,
	/// Whether and how to generate index/barrel files.
	pub index_files: IndexFileConfig,
	/// How cross-module imports are written (language-specific behavior
	/// is in the trait, but this controls the structural style).
	pub import_style: ImportStyle,
}
```
