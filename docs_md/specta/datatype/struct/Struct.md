# Struct `Struct` -- path: `specta::datatype::struct::Struct`

represents a Rust [struct](https://doc.rust-lang.org/std/keyword.struct.html).

```rust
pub struct Struct {
	fields: crate::datatype::Fields,
	attributes: crate::datatype::Attributes,
}
```

## Methods

```rust
/**
`unit` -- Construct a new unit struct.
*/
pub fn unit() -> Self
/**
`named` -- Construct a named struct.
*/
pub fn named() -> StructBuilder<NamedFields>
/**
`unnamed` -- Construct an unnamed struct.
*/
pub fn unnamed() -> StructBuilder<UnnamedFields>
/**
`fields` -- Get a immutable reference to the fields of the struct.
*/
pub fn fields(self: &Self) -> &Fields
/**
`fields_mut` -- Get a mutable reference to the fields of the struct.
*/
pub fn fields_mut(self: &mut Self) -> &mut Fields
/**
`set_fields` -- Set the fields of the struct.
*/
pub fn set_fields(self: &mut Self, fields: Fields)
/**
`attributes` -- Get a immutable reference to the attributes of the struct.
*/
pub fn attributes(self: &Self) -> &Attributes
/**
`attributes_mut` -- Get a mutable reference to the attributes of the struct.
*/
pub fn attributes_mut(self: &mut Self) -> &mut Attributes
```

