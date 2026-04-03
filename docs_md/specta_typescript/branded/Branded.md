# Struct `Branded` -- path: `specta_typescript::branded::Branded`

Runtime payload for a TypeScript branded type.

```rust
pub struct Branded {
	brand: std::borrow::Cow<''static, str>,
	ty: specta::datatype::DataType,
}
```

## Methods

```rust
/**
`new` -- Construct a branded type from a brand label and inner type.
*/
pub fn new<impl Into<Cow<'static, str>>>(brand: impl , ty: DataType) -> Self
/**
`brand` -- Get the brand label.
*/
pub fn brand(self: &Self) -> &Cow<''static, str>
/**
`ty` -- Get the inner data type.
*/
pub fn ty(self: &Self) -> &DataType
```

