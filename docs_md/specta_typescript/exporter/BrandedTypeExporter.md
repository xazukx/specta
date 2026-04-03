# Struct `BrandedTypeExporter` -- path: `specta_typescript::exporter::BrandedTypeExporter`

Reference to Typescript language exporter for branded type callbacks.

```rust
pub struct BrandedTypeExporter {
	exporter: &Exporter,
	/// Collected types currently being exported.
	pub types: &specta::ResolvedTypes,
}
```

## Methods

```rust
/**
`inline` -- [primitives::inline]
*/
pub fn inline(self: &Self, dt: &DataType) -> Result<String, Error>
/**
`reference` -- [primitives::reference]
*/
pub fn reference(self: &Self, r: &Reference) -> Result<String, Error>
```

