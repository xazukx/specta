# Enum `ConstantValue` -- path: `specta::datatype::constant::ConstantValue`

A compile-time constant value that can be exported by language exporters.

This represents the runtime value of a Rust constant, allowing exporters
to emit `export const X = ... as const;` style declarations.

```rust
pub enum ConstantValue {
	/// A string constant.
	String(std::borrow::Cow<''static, str>),
	/// A signed integer constant.
	Integer(i128),
	/// An unsigned integer constant.
	UnsignedInteger(u128),
	/// A floating-point constant stored as raw bits for Eq/Hash compatibility.
	/// [FloatBits](./FloatBits.md)
	Float(FloatBits),
	/// A boolean constant.
	Bool(bool),
	/// A byte array constant.
	Bytes(std::borrow::Cow<''static, [u8]>),
	/// A null/unit constant.
	Null,
}
```
