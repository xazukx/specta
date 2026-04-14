# Struct `IndexFileEntry` -- path: `specta::export::IndexFileEntry`

An entry in an index/barrel file.

```rust
pub struct IndexFileEntry {
	/// File stem (without extension), e.g., `"item"` for `item.ts`.
	pub file_stem: &str,
	/// Names exported by this file.
	pub exported_names: &[String],
	/// The module path this file corresponds to.
	pub module_path: &str,
}
```
