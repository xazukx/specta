# Struct `UnnamedFields` -- path: `specta::datatype::fields::UnnamedFields`

The fields of an unnamed enum variant.

```rust
pub struct UnnamedFields {
	fields: Vec<Field>,
}
```

## Methods

```rust
/**
`fields` -- Get an immutable reference to the fields of this unnamed enum variant.
*/
pub fn fields(self: &Self) -> &[Field]
/**
`fields_mut` -- Mutable reference to the fields of this unnamed enum variant.
*/
pub fn fields_mut(self: &mut Self) -> &mut Vec<Field>
```

