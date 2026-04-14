# Struct `Exporter` -- path: `specta_typescript::exporter::Exporter`

Typescript language exporter.

```rust
pub struct Exporter {
	/// Custom header prepended to exported files.
	pub header: std::borrow::Cow<''static, str>,
	framework_runtime: Option<RuntimeFn>,
	branded_type_impl: Option<BrandedTypeImpl>,
	framework_prelude: std::borrow::Cow<''static, str>,
	/// Output layout mode for generated TypeScript.
	pub layout: specta::export::Layout,
	/// The export mode (TypeScript, JSDoc, or Zod).
	mode: ExportMode,
}
```

## Methods

```rust
/**
`framework_prelude` -- Provide a prelude which is added to the start of all exported files.
*/
pub fn framework_prelude<impl Into<Cow<'static, str>>>(self: Self, prelude: impl ) -> Self
/**
`framework_runtime` -- Add some custom Typescript or Javascript code that is exported as part of the bindings.
It's appending to the types file for single-file layouts or put in a root `index.{ts/js}` for multi-file.

The closure is wrapped in [`specta::collect_types()`] to capture any referenced types.
Ensure you call `T::reference()` within the closure if you want an import to be created.
*/
pub fn framework_runtime<impl Fn(FrameworkExporter) -> Result<Cow<'static, str>, Error> + Send + Sync + 'static>(self: Self, builder: impl ) -> Self
/**
`branded_type_impl` -- Configure how `specta_typescript::branded!` types are rendered with exporter context.

This callback receives both the branded payload and a [`BrandedTypeExporter`], allowing
you to call [`BrandedTypeExporter::inline`] / [`BrandedTypeExporter::reference`] while
preserving the current export configuration.

# Examples

`ts-brand` style:
```rust
# use std::borrow::Cow;
# use specta_typescript::{Branded, Error, Typescript};
let exporter = Typescript::default().branded_type_impl(|ctx, branded| {
    let datatype = ctx.inline(branded.ty())?;

    Ok(Cow::Owned(format!(
        "import(\"ts-brand\").Brand<{}, \"{}\">",
        datatype,
        branded.brand()
    )))
});
# let _ = exporter;
```

Effect style:
```rust
# use std::borrow::Cow;
# use specta_typescript::{Branded, Error, Typescript};
let exporter = Typescript::default().branded_type_impl(|ctx, branded| {
    let datatype = ctx.inline(branded.ty())?;

    Ok(Cow::Owned(format!(
        "{} & import(\"effect\").Brand.Brand<\"{}\">",
        datatype,
        branded.brand()
    )))
});
# let _ = exporter;
```
*/
pub fn branded_type_impl<impl for<'a> Fn(BrandedTypeExporter<'a>, &Branded) -> Result<Cow<'static, str>, Error> + Send + Sync + 'static>(self: Self, builder: impl ) -> Self
/**
`header` -- Configure a header for the file.

This is perfect for configuring lint ignore rules or other file-level comments.
*/
pub fn header<impl Into<Cow<'static, str>>>(self: Self, header: impl ) -> Self
/**
`layout` -- Configure the bindings layout
*/
pub fn layout(self: Self, layout: Layout) -> Self
/**
`always_use_number` -- Export bigint types (`i64`, `u64`, `i128`, `u128`, `isize`, `usize`, `f128`) as `number`
instead of returning an error. Only applies to TypeScript/JSDoc modes.
*/
pub fn always_use_number(self: Self, enable: bool) -> Self
/**
`namespaces` -- Enable TypeScript namespace wrapping for single-file output.

When enabled, types are grouped into `namespace` blocks matching their
Rust module paths. Only applies to `Layout::SingleFile` layouts in TypeScript mode.
*/
pub fn namespaces(self: Self, enable: bool) -> Self
/**
`export` -- Export the files into a single string.

Note: This returns an error if the layout is `Layout::MultiFile`.
*/
pub fn export(self: &Self, resolved_types: &ResolvedTypes) -> Result<String, Error>
/**
`export_to` -- Export the types to a specific file/folder.

When configured when `format` is `Format::Files`, you must provide a directory path.
Otherwise, you must provide the path of a single file.

*/
pub fn export_to<impl AsRef<Path>>(self: &Self, path: impl , resolved_types: &ResolvedTypes) -> Result<(), Error>
```

