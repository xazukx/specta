# Struct `ZodConfig` -- path: `specta_typescript::exporter::ZodConfig`

Zod-specific export configuration.

```rust
pub struct ZodConfig {
	/// Target Zod version for generated schemas.
	pub zod_version: ZodVersion,
	/// Strategy for exporting Rust bigint-compatible primitives.
	pub bigint: BigIntExportBehavior,
	/// Whether to export inferred TypeScript types (`export type X = z.infer<...>`).
	pub output_type_infers: bool,
}
```
