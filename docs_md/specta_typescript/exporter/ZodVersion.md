# Enum `ZodVersion` -- path: `specta_typescript::exporter::ZodVersion`

Target Zod version for generated schemas.

```rust
pub enum ZodVersion {
	/// Zod v3 — stable, backward-compatible output.
	V3,
	/// Zod v4 — uses `z.strictObject()`, `z.int()`, and other v4-specific APIs.
	V4,
}
```
