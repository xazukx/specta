# Struct `NamedFields` -- path: `specta::datatype::fields::NamedFields`

The fields of an named enum variant or a struct.

```rust
pub struct NamedFields {
	fields: Vec<(std::borrow::Cow<''static, str>, Field)>,
}
```

## Methods

```rust
/**
`fields` -- Get an immutable reference to the fields.
*/
pub fn fields(self: &Self) -> &[(Cow<''static, str>, Field)]
/**
`fields_mut` -- Mutable reference to the fields.
*/
pub fn fields_mut(self: &mut Self) -> &mut Vec<(Cow<''static, str>, Field)>
```

