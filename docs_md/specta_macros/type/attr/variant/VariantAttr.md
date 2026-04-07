# Struct `VariantAttr` -- path: `specta_macros::type::attr::variant::VariantAttr`

```rust
pub struct VariantAttr {
	pub type: Option<syn::Type>,
	pub skip: bool,
	pub inline: bool,
	pub common: super::RustCAttr,
}
```

## Methods

```rust

pub fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self>
```

