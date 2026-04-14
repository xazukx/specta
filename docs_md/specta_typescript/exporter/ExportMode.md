# Enum `ExportMode` -- path: `specta_typescript::exporter::ExportMode`

The export mode determines which language dialect is generated.

```rust
pub enum ExportMode {
	/// Standard TypeScript type declarations (`.ts` files).
	/// [TypescriptConfig](./TypescriptConfig.md)
	Typescript(TypescriptConfig),
	/// JSDoc typedef annotations (`.js` files).
	/// [JSDocConfig](./JSDocConfig.md)
	JSDoc(JSDocConfig),
	/// Zod schema declarations (`.ts` files).
	/// [ZodConfig](./ZodConfig.md)
	Zod(ZodConfig),
}
```
