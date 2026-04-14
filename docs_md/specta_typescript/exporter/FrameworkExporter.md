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
`render_types` -- Render the types within the [`ResolvedTypes`].

This will only work if used within [`Exporter::framework_runtime`].
It allows frameworks to intersperse their user types into their runtime code.
*/
pub fn render_types(self: &mut Self) -> Result<Cow<''static, str>, Error>
/**
`inline` -- Inline a single [`DataType`] expression.
*/
pub fn inline(self: &Self, dt: &DataType) -> Result<String, Error>
/**
`reference` -- Render a [`Reference`] expression.
*/
pub fn reference(self: &Self, r: &Reference) -> Result<String, Error>
/**
`export` -- Export a group of [`NamedDataType`] declarations.
*/
pub fn export<'a, impl Iterator<Item = &'a NamedDataType>>(self: &Self, ndts: impl , indent: &str) -> Result<String, Error>
```

