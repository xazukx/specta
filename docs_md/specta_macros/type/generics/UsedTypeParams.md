# Struct `UsedTypeParams` -- path: `specta_macros::type::generics::UsedTypeParams`

```rust
pub struct UsedTypeParams {
	pub direct: Vec<syn::Ident>,
	pub associated: Vec<syn::TypePath>,
	pub conservative: bool,
}
```
