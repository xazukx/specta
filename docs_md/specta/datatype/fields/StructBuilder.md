# Struct `StructBuilder` -- path: `specta::datatype::fields::StructBuilder`

Builder for constructing [`DataType::Struct`] values.

```rust
pub struct StructBuilder {
	fields: F,
}
```

## Methods

```rust
/**
`field` -- Add a named field.
*/
pub fn field<impl Into<Cow<'static, str>>>(self: Self, name: impl , field: Field) -> Self
/**
`field_mut` -- Add a named field in-place.
*/
pub fn field_mut<impl Into<Cow<'static, str>>>(self: &mut Self, name: impl , field: Field)
/**
`build` -- Finalize this builder into a [`DataType`].
*/
pub fn build(self: Self) -> DataType
/**
`field` -- Add an unnamed field.
*/
pub fn field(self: Self, field: Field) -> Self
/**
`field_mut` -- Add an unnamed field in-place.
*/
pub fn field_mut(self: &mut Self, field: Field)
/**
`build` -- Finalize this builder into a [`DataType`].
*/
pub fn build(self: Self) -> DataType
```

