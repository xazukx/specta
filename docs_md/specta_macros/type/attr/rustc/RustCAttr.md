# Struct `RustCAttr` -- path: `specta_macros::type::attr::rustc::RustCAttr`

```rust
pub struct RustCAttr {
	pub doc: String,
	pub deprecated: Option<Deprecated>,
}
```

## Methods

```rust

pub fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self>

pub fn deprecated_as_tokens(self: &Self) -> proc_macro2::TokenStream
```

