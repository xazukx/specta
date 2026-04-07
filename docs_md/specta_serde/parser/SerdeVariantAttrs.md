# Struct `SerdeVariantAttrs` -- path: `specta_serde::parser::SerdeVariantAttrs`

```rust
pub struct SerdeVariantAttrs {
	pub rename_serialize: Option<String>,
	pub rename_deserialize: Option<String>,
	pub aliases: Vec<String>,
	pub rename_all_serialize: Option<crate::inflection::RenameRule>,
	pub rename_all_deserialize: Option<crate::inflection::RenameRule>,
	pub skip_serializing: bool,
	pub skip_deserializing: bool,
	pub has_serialize_with: bool,
	pub has_deserialize_with: bool,
	pub has_with: bool,
	pub other: bool,
	pub untagged: bool,
}
```

## Methods

```rust

pub fn from_attributes(attributes: &Attributes) -> Result<Option<Self>>
```

