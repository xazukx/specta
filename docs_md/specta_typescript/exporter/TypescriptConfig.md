# Struct `TypescriptConfig` -- path: `specta_typescript::exporter::TypescriptConfig`

TypeScript-specific export configuration.

```rust
pub struct TypescriptConfig {
	/// When `true`, wrap output in TypeScript namespaces (single-file layouts only).
	pub use_namespaces: bool,
	/// When `true`, bigint types are exported as `number` instead of returning an error.
	pub always_use_number: bool,
}
```
