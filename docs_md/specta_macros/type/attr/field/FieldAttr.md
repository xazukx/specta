# Struct `FieldAttr` -- path: `specta_macros::type::attr::field::FieldAttr`

```rust
pub struct FieldAttr {
	pub type: Option<syn::Type>,
	pub inline: bool,
	pub skip: bool,
	pub optional: bool,
	pub common: super::RustCAttr,
}
```

## Methods

```rust

pub fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self>
```

