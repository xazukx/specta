# Enum `IndexFileConfig` -- path: `specta::export::layout::IndexFileConfig`

Controls whether and how index/barrel files are generated.

```rust
pub enum IndexFileConfig {
	/// Do not generate index/barrel files.
	None,
	/// Generate an index file in each folder that re-exports
	/// everything from that folder's files.
	ReExportAll,
}
```
