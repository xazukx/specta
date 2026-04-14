# Struct `JSDoc` -- path: `specta_typescript::jsdoc::JSDoc`

JSDoc language exporter.

```rust
pub struct JSDoc (
	crate::Exporter,
)
```

## Methods

```rust
/**
`new` -- Construct a new JSDoc exporter with the default options configured.
*/
pub fn new() -> Self
/**
`header` -- Configure a header for the file.

This is perfect for configuring lint ignore rules or other file-level comments.
*/
pub fn header<impl Into<Cow<'static, str>>>(self: Self, header: impl ) -> Self
/**
`layout` -- Configure the layout of the generated file
*/
pub fn layout(self: Self, layout: Layout) -> Self
/**
`always_use_number` -- Export bigint types (`i64`, `u64`, `i128`, `u128`, `isize`, `usize`, `f128`) as `number`
instead of returning an error.
*/
pub fn always_use_number(self: Self, enable: bool) -> Self
/**
`namespaces` -- Enable TypeScript namespace wrapping for single-file output.
*/
pub fn namespaces(self: Self, enable: bool) -> Self
/**
`branded_type_impl` -- Configure how `specta_typescript::branded!` types are rendered.

See [`Exporter::branded_type_impl`] for details.
*/
pub fn branded_type_impl<impl for<'a> Fn(BrandedTypeExporter<'a>, &Branded) -> Result<Cow<'static, str>, Error> + Send + Sync + 'static>(self: Self, builder: impl ) -> Self
/**
`export` -- Export the files into a single string.

Note: This returns an error if the format is `Format::Files`.
*/
pub fn export(self: &Self, types: &ResolvedTypes) -> Result<String, Error>
/**
`export_to` -- Export the types to a specific file/folder.

When configured when `format` is `Format::Files`, you must provide a directory path.
Otherwise, you must provide the path of a single file.

*/
pub fn export_to<impl AsRef<Path>>(self: &Self, path: impl , types: &ResolvedTypes) -> Result<(), Error>
```

