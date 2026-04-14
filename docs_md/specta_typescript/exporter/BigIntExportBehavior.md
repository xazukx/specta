# Enum `BigIntExportBehavior` -- path: `specta_typescript::exporter::BigIntExportBehavior`

Configures how the exporter deals with BigInt types ([i64], [i128] etc) in Zod mode.

```rust
pub enum BigIntExportBehavior {
	/// Export BigInt as a Zod string schema.
	String,
	/// Export BigInt as a Zod number schema.
	Number,
	/// Export BigInt as a Zod bigint schema.
	BigInt,
	/// Abort export on BigInt usage.
	Fail,
}
```
