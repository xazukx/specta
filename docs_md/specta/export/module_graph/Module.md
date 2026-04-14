# Struct `Module` -- path: `specta::export::module_graph::Module`

A node in the module tree. Represents a Rust module that contains
types, constants, and child modules.

```rust
pub struct Module {
	/// Types belonging directly to this module.
	pub types: Vec<&crate::datatype::NamedDataType>,
	/// Constants belonging directly to this module.
	pub constants: Vec<&crate::NamedConstant>,
	/// Child modules keyed by segment name.
	pub children: std::collections::BTreeMap<&str, Module<''a>>,
	/// Full Rust module path (e.g., `"shared::item"`).
	pub module_path: std::borrow::Cow<''static, str>,
}
```
