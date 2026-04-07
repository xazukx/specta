# Functions in specta_macros::utils

```rust
/**
`parse_attrs` -- pass all of the attributes into a single structure.
We can then remove them from the struct while passing an any left over must be invalid and an error can be thrown.
*/
pub fn parse_attrs(attrs: &[syn::Attribute]) -> syn::Result<Vec<Attribute>>
/**
`parse_attrs_with_filter` -- Same as `parse_attrs` but allows skipping attributes by name.
This is useful for skipping attributes that may have non-standard syntax that we can't parse.
*/
pub fn parse_attrs_with_filter(attrs: &[syn::Attribute], skip_attrs: &[String]) -> syn::Result<Vec<Attribute>>

pub fn unraw_raw_ident(ident: &syn::Ident) -> String
```

