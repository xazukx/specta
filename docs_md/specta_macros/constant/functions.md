# Functions in specta_macros::constant

```rust
/**
`attribute` -- `#[specta_const]` attribute macro for bare `const` items.

Generates a hidden struct + `Constant` impl alongside the original const.
*/
pub fn attribute(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> syn::Result<proc_macro::TokenStream>
```

