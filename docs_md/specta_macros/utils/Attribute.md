# Struct `Attribute` -- path: `specta_macros::utils::Attribute`

```rust
pub struct Attribute {
	/// Source of the attribute. Eg. `specta`, `serde`, `repr`, `deprecated`, etc.
	pub source: String,
	/// Key of the current item. Eg. `specta` or `type`in `#[specta(type = String)]`
	pub key: syn::Ident,
	/// Value of the item. Eg. `String` in `#[specta(type = String)]`
	pub value: Option<AttributeValue>,
}
```

## Methods

```rust
/**
`value_span` -- Span of they value. Eg. `String` in `#[specta(type = String)]`
Will fallback to the key span if no value is present.
*/
pub fn value_span(self: &Self) -> Span

pub fn parse_string(self: &Self) -> Result<String>

pub fn parse_bool(self: &Self) -> Result<bool>

pub fn parse_path(self: &Self) -> Result<Path>

pub fn parse_type(self: &Self) -> Result<Type>
```

