# Functions in specta_macros::type::generics

```rust

pub fn add_type_to_where_clause(ty: &proc_macro2::TokenStream, generics: &syn::Generics, custom_bounds: Option<&[syn::WherePredicate]>, used_generic_types: &[syn::Ident], associated_type_usage: &[syn::TypePath]) -> Option<syn::WhereClause>

pub fn all_type_param_idents(generics: &syn::Generics) -> Vec<syn::Ident>

pub fn generics_with_ident_and_bounds_only(generics: &syn::Generics) -> Option<proc_macro2::TokenStream>

pub fn generics_with_ident_only(generics: &syn::Generics) -> Option<proc_macro2::TokenStream>

pub fn has_associated_type_usage(used_generic_types: &UsedTypeParams) -> bool

pub fn used_associated_type_paths(used_generic_types: &UsedTypeParams) -> &[syn::TypePath]

pub fn used_direct_type_params<'a>(used_generic_types: &UsedTypeParams, all_generic_type_idents: &[syn::Ident]) -> &[syn::Ident]

pub fn used_type_params(generics: &syn::Generics, data: &syn::Data, container_type: Option<&syn::Type>) -> UsedTypeParams
```

