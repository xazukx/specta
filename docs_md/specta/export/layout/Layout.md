# Enum `Layout` -- path: `specta::export::layout::Layout`

Controls how generated types are organized on disk.
Shared across all exporter crates.

```rust
pub enum Layout {
	/// All types in a single file.
	/// [SingleFileConfig](./SingleFileConfig.md)
	SingleFile(SingleFileConfig),
	/// One file per Rust module path. Cross-module references use
	/// language-specific import/reference mechanisms.
	/// [MultiFileConfig](./MultiFileConfig.md)
	MultiFile(MultiFileConfig),
}
```

## Methods

```rust
/**
`multi_file` -- Shorthand for `Layout::MultiFile(MultiFileConfig::default())`.
*/
pub fn multi_file() -> Self
/**
`module_prefixed` -- Shorthand for `Layout::SingleFile(SingleFileConfig { module_prefix_names: true })`.
*/
pub fn module_prefixed() -> Self
```

