# Enum `FolderGrouping` -- path: `specta::export::layout::FolderGrouping`

Controls how module paths are grouped into filesystem folders.

```rust
pub enum FolderGrouping {
	/// No folder grouping; current behavior. Each module path segment
	/// maps to a nested file. `shared::item` -> `shared/item.{ext}`
	/// with `shared.{ext}` for types directly in `shared`.
	None,
	/// Group by the first N segments of the module path into folders.
	/// With depth=1: `shared::item` and `shared::item::data` both go
	/// into the `shared/` folder. `tools::declarations` goes into `tools/`.
	ByDepth(usize),
}
```
