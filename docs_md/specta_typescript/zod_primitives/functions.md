# Functions in specta_typescript::zod_primitives

```rust
/**
`export` -- Generate a group of `export const XSchema = ...` declarations for named types.
*/
pub fn export<'a, impl Iterator<Item = &'a NamedDataType>>(exporter: &dyn AsRef, types: &specta::ResolvedTypes, ndts: impl , indent: &str) -> Result<String, crate::Error>
/**
`inline` -- Generate an inline Zod expression for a [`DataType`].
*/
pub fn inline(exporter: &dyn AsRef, types: &specta::ResolvedTypes, dt: &specta::datatype::DataType) -> Result<String, crate::Error>
/**
`reference` -- Generate a Zod expression for a [`Reference`].
*/
pub fn reference(exporter: &dyn AsRef, types: &specta::ResolvedTypes, r: &specta::datatype::Reference) -> Result<String, crate::Error>
```

