# Enum `ConstantOutput` -- path: `specta_macros::constant::attr::ConstantOutput`

Output format for the constant value in TypeScript.

```rust
pub enum ConstantOutput {
	/// Infer the output format from the Rust type (default).
	Infer,
	/// Force output as a string literal.
	String,
	/// Force output as a byte array.
	Bytes,
}
```
