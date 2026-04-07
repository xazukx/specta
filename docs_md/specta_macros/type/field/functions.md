# Functions in specta_macros::type::field

```rust

pub fn construct_field(crate_ref: &proc_macro2::TokenStream, container_attrs: &super::ContainerAttr, attrs: super::FieldAttr, field_ty: &syn::Type, raw_attrs: &[syn::Attribute]) -> syn::Result<proc_macro2::TokenStream>

pub fn construct_field_with_variant_skip(crate_ref: &proc_macro2::TokenStream, container_attrs: &super::ContainerAttr, attrs: super::FieldAttr, field_ty: &syn::Type, raw_attrs: &[syn::Attribute], variant_skip: bool) -> syn::Result<proc_macro2::TokenStream>
```

