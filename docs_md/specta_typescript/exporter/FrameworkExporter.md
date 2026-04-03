# Struct `FrameworkExporter` -- path: `specta_typescript::exporter::FrameworkExporter`

Reference to Typescript language exporter for framework

```rust
pub struct FrameworkExporter {
	exporter: &Exporter,
	has_manually_exported_user_types: &mut bool,
	files_root_types: &str,
	/// Collected types currently being exported.
	pub types: &specta::ResolvedTypes,
}
```

## Methods

```rust
/**
`render_types` -- Render the types within the [`ResolvedTypes`](specta::ResolvedTypes).

This will only work if used within [`Exporter::framework_runtime`].
It allows frameworks to intersperse their user types into their runtime code.
*/
pub fn render_types(self: &mut Self) -> Result<Cow<''static, str>, Error>
/**
`inline` -- [primitives::inline]
*/
pub fn inline(self: &Self, dt: &DataType) -> Result<String, Error>
/**
`reference` -- [primitives::reference]
*/
pub fn reference(self: &Self, r: &Reference) -> Result<String, Error>
/**
`export` -- [primitives::export]
*/
pub fn export<'a, impl Iterator<Item = &'a NamedDataType>>(self: &Self, ndts: impl , indent: &str) -> Result<String, Error>
```

