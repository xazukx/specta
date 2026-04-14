# Struct `Zod` -- path: `specta_typescript::zod::Zod`

Zod schema exporter.

Produces Zod schema declarations (`export const XSchema = z.object({...})`)
instead of TypeScript type declarations. Output files are `.ts`.

```rust
pub struct Zod (
	crate::Exporter,
)
```

## Methods

```rust
/**
`new` -- Construct a new Zod exporter with default options configured.
*/
pub fn new() -> Self
/**
`v3` -- Construct a Zod v3 exporter.
*/
pub fn v3() -> Self
/**
`header` -- Configure a header for the file.

This is perfect for configuring lint ignore rules or other file-level comments.
*/
pub fn header<impl Into<Cow<'static, str>>>(self: Self, header: impl ) -> Self
/**
`layout` -- Configure the layout of the generated file.
*/
pub fn layout(self: Self, layout: Layout) -> Self
/**
`bigint` -- Configure BigInt handling behaviour.
*/
pub fn bigint(self: Self, bigint: BigIntExportBehavior) -> Self
/**
`zod_version` -- Configure the target Zod version.
*/
pub fn zod_version(self: Self, version: ZodVersion) -> Self
/**
`output_type_infers` -- Configure whether to export inferred TypeScript types (`export type X = z.infer<...>`).
*/
pub fn output_type_infers(self: Self, export: bool) -> Self
/**
`framework_prelude` -- Provide a prelude which is added to the start of all exported files.
*/
pub fn framework_prelude<impl Into<Cow<'static, str>>>(self: Self, prelude: impl ) -> Self
/**
`framework_runtime` -- Add runtime code exported as part of the bindings.
*/
pub fn framework_runtime<impl Fn(crate::FrameworkExporter<'_>) -> Result<Cow<'static, str>, Error> + Send + Sync + 'static>(self: Self, builder: impl ) -> Self
/**
`export` -- Export the files into a single string.

Note: This returns an error if the layout is `Layout::MultiFile`.
*/
pub fn export(self: &Self, types: &ResolvedTypes) -> Result<String, Error>
/**
`export_to` -- Export the types to a specific file/folder.

When configured with `Layout::MultiFile`, you must provide a directory path.
Otherwise, you must provide the path of a single file.
*/
pub fn export_to<impl AsRef<Path>>(self: &Self, path: impl , types: &ResolvedTypes) -> Result<(), Error>
```

