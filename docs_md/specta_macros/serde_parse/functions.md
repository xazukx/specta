# Functions in specta_macros::serde_parse

```rust

pub fn parse_container_attrs(attrs: &[syn::Attribute]) -> syn::Result<Option<ContainerAttrs>>

pub fn parse_field_attrs(attrs: &[syn::Attribute]) -> syn::Result<Option<FieldAttrs>>

pub fn parse_field_meta(target: &mut FieldAttrs, meta: syn::meta::ParseNestedMeta<''_>) -> syn::Result<()>
/**
`parse_lit_str` -- Parse `= "literal"` from a meta item and return the `LitStr`.
*/
pub fn parse_lit_str(meta: &syn::meta::ParseNestedMeta<''_>) -> syn::Result<syn::LitStr>
/**
`parse_string_assignment` -- Parse `= "literal"` from a meta item and return the string value.
*/
pub fn parse_string_assignment(meta: &syn::meta::ParseNestedMeta<''_>) -> syn::Result<String>

pub fn parse_variant_attrs(attrs: &[syn::Attribute]) -> syn::Result<Option<VariantAttrs>>

pub fn parse_variant_meta(target: &mut VariantAttrs, meta: syn::meta::ParseNestedMeta<''_>) -> syn::Result<()>
/**
`resolve_serialized_name` -- Resolve the serialized name for a field or variant.

Priority:
1. Explicit `rename_serialize` on the field/variant
2. `rename_all_serialize` on the container, applied to the Rust name
3. The Rust name as-is
*/
pub fn resolve_serialized_name(rust_name: &str, rename_serialize: Option<&str>, container_rename_all: Option<RenameRule>) -> String
/**
`skip_meta_value` -- Consume remaining value or nested group from an unrecognized meta attribute.

When `parse_nested_meta` encounters an unknown attribute like
`skip_serializing_if = "Option::is_none"`, the callback must consume the
`= "..."` portion. Otherwise the parser will report "expected ," because
the unconsumed tokens remain in the stream.
*/
pub fn skip_meta_value(meta: &syn::meta::ParseNestedMeta<''_>)
/**
`to_pascal_case` -- Convert a string to PascalCase for use as an enum variant name.
*/
pub fn to_pascal_case(s: &str) -> String
```

