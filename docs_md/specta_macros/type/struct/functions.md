# Functions in specta_macros::type::struct

```rust

pub fn decode_field_attrs<'a>(field: &syn::Field, skip_attrs: &[String]) -> syn::Result<(FieldAttr, &[syn::Attribute])>

pub fn parse_struct(crate_ref: &proc_macro2::TokenStream, container_attrs: &ContainerAttr, data: &syn::DataStruct) -> syn::Result<proc_macro2::TokenStream>
```

