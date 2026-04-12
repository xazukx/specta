# Struct `ContainerAttrs` -- path: `specta_macros::serde_parse::ContainerAttrs`

```rust
pub struct ContainerAttrs {
	pub rename_serialize: Option<String>,
	pub rename_deserialize: Option<String>,
	pub rename_all_serialize: Option<RenameRule>,
	pub rename_all_deserialize: Option<RenameRule>,
	pub rename_all_fields_serialize: Option<RenameRule>,
	pub rename_all_fields_deserialize: Option<RenameRule>,
	pub tag: Option<String>,
	pub content: Option<String>,
	pub untagged: bool,
	pub default: bool,
	pub transparent: bool,
	pub from: Option<ConversionType>,
	pub try_from: Option<ConversionType>,
	pub into: Option<ConversionType>,
	pub variant_identifier: bool,
	pub field_identifier: bool,
}
```
