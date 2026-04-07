# Struct `SerdeFieldAttrs` -- path: `specta_serde::parser::SerdeFieldAttrs`

```rust
pub struct SerdeFieldAttrs {
	pub rename_serialize: Option<String>,
	pub rename_deserialize: Option<String>,
	pub aliases: Vec<String>,
	pub default: bool,
	pub flatten: bool,
	pub skip_serializing: bool,
	pub skip_deserializing: bool,
	pub skip_serializing_if: Option<String>,
	pub has_serialize_with: bool,
	pub has_deserialize_with: bool,
	pub has_with: bool,
}
```

## Methods

```rust

pub fn from_attributes(attributes: &Attributes) -> Result<Option<Self>>
```

