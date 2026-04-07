# Struct `ContainerAttr` -- path: `specta_macros::type::attr::container::ContainerAttr`

```rust
pub struct ContainerAttr {
	pub type: Option<syn::Type>,
	pub crate_name: Option<proc_macro2::TokenStream>,
	pub inline: bool,
	pub remote: Option<proc_macro2::TokenStream>,
	pub collect: Option<bool>,
	pub skip_attrs: Vec<String>,
	pub common: super::RustCAttr,
	pub transparent: bool,
	pub ts_enum: bool,
	pub bound: Option<Vec<syn::WherePredicate>>,
}
```

## Methods

```rust

pub fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self>
```

