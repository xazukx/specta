# Functions in specta_typescript::primitives

```rust
/**
`export` -- Generate a group of `export Type = ...` Typescript string for a specific [`NamedDataType`].

This method leaves the following up to the implementer:
 - Ensuring all referenced types are exported
 - Handling multiple type with overlapping names
 - Transforming the type for your serialization format (Eg. Serde)

We recommend passing in your types in bulk instead of doing individual calls as it leaves formatting to us and also allows us to merge the JSDoc types into a single large comment.

*/
pub fn export<'a, impl Iterator<Item = &'a NamedDataType>>(exporter: &dyn AsRef, types: &specta::ResolvedTypes, ndts: impl , indent: &str) -> Result<String, crate::Error>
/**
`inline` -- Generate an inlined Typescript string for a specific [`DataType`].

This methods leaves all the same things as the [`export`] method up to the user.

Note that calling this method with a tagged struct or enum may cause the tag to not be exported.
The type should be wrapped in a [`NamedDataType`] to provide a proper name.

*/
pub fn inline(exporter: &dyn AsRef, types: &specta::ResolvedTypes, dt: &specta::datatype::DataType) -> Result<String, crate::Error>
/**
`reference` -- Generate an Typescript string to refer to a specific [`DataType`].

For primitives this will include the literal type but for named type it will contain a reference.

See [`export`] for the list of things to consider when using this.
*/
pub fn reference(exporter: &dyn AsRef, types: &specta::ResolvedTypes, r: &specta::datatype::Reference) -> Result<String, crate::Error>
```

