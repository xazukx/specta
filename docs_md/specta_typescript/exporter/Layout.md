# Enum `Layout` -- path: `specta_typescript::exporter::Layout`

Allows configuring the format of the final types file

```rust
pub enum Layout {
	/// Produce a Typescript namespace for each Rust module
	Namespaces,
	/// Produce a dedicated file for each Rust module
	Files,
	/// Include the full module path in the types name but keep a flat structure.
	ModulePrefixedName,
	/// Flatten all of the types into a single file of types.
	/// This mode doesn't support having multiple types with the same name.
	FlatFile,
}
```
