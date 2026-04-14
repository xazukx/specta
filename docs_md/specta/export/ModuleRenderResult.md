# Struct `ModuleRenderResult` -- path: `specta::export::ModuleRenderResult`

Result of rendering a module's types.

```rust
pub struct ModuleRenderResult {
	/// The rendered type declarations as a string.
	pub body: String,
	/// Map of exported type names to their source locations (for duplicate detection).
	pub exports: std::collections::HashMap<String, std::panic::Location<''static>>,
	/// Set of module paths that this module's types reference.
	/// Maps module_path -> import information including type names and value flags.
	pub referenced_modules: std::collections::BTreeMap<String, ImportInfo>,
}
```
