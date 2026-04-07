# Struct `NamedConstant` -- path: `specta::constant::NamedConstant`

A named constant that can be exported by language exporters.

```rust
pub struct NamedConstant {
	/// Export name (the TypeScript variable name).
	pub name: std::borrow::Cow<''static, str>,
	/// The resolved constant value.
	pub value: crate::datatype::ConstantValue,
	/// Documentation comments.
	pub docs: std::borrow::Cow<''static, str>,
	/// Deprecation metadata.
	pub deprecated: Option<crate::datatype::Deprecated>,
	/// Source module path.
	pub module_path: std::borrow::Cow<''static, str>,
}
```
