# Struct `VariantAttrs` -- path: `specta_macros::serde_parse::VariantAttrs`

```rust
pub struct VariantAttrs {
	pub rename_serialize: Option<String>,
	pub rename_deserialize: Option<String>,
	pub aliases: Vec<String>,
	pub rename_all_serialize: Option<RenameRule>,
	pub rename_all_deserialize: Option<RenameRule>,
	pub skip_serializing: bool,
	pub skip_deserializing: bool,
	pub has_serialize_with: bool,
	pub has_deserialize_with: bool,
	pub has_with: bool,
	pub other: bool,
	pub untagged: bool,
}
```
