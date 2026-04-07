# Struct `SerdeContainerAttrs` -- path: `specta_serde::parser::SerdeContainerAttrs`

```rust
pub struct SerdeContainerAttrs {
	pub rename_serialize: Option<String>,
	pub rename_deserialize: Option<String>,
	pub rename_all_serialize: Option<crate::inflection::RenameRule>,
	pub rename_all_deserialize: Option<crate::inflection::RenameRule>,
	pub rename_all_fields_serialize: Option<crate::inflection::RenameRule>,
	pub rename_all_fields_deserialize: Option<crate::inflection::RenameRule>,
	pub tag: Option<String>,
	pub content: Option<String>,
	pub untagged: bool,
	pub default: bool,
	pub transparent: bool,
	pub from: Option<ConversionType>,
	pub try_from: Option<ConversionType>,
	pub into: Option<ConversionType>,
	pub resolved_from: Option<specta::datatype::DataType>,
	pub resolved_try_from: Option<specta::datatype::DataType>,
	pub resolved_into: Option<specta::datatype::DataType>,
	pub variant_identifier: bool,
	pub field_identifier: bool,
}
```

## Methods

```rust

pub fn from_attributes(attributes: &Attributes) -> Result<Option<Self>>
```

