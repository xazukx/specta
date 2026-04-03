# Struct `Enum` -- path: `specta::datatype::enum::Enum`

represents a Rust [enum](https://doc.rust-lang.org/std/keyword.enum.html).

Enums are configured with a set of variants, each with a name and a type.
The variants can be either unit variants (no fields), tuple variants (fields in a tuple), or struct variants (fields in a struct).

An enum is also assigned a repr which follows [Serde repr semantics](https://serde.rs/enum-representations.html).

```rust
pub struct Enum {
	variants: Vec<(std::borrow::Cow<''static, str>, Variant)>,
	attributes: super::Attributes,
}
```

## Methods

```rust
/**
`new` -- Construct a new empty enum.
*/
pub fn new() -> Self
/**
`variants` -- Get an immutable reference to the enum's variants.
*/
pub fn variants(self: &Self) -> &[(Cow<''static, str>, Variant)]
/**
`variants_mut` -- Get a mutable reference to the enum's variants.
*/
pub fn variants_mut(self: &mut Self) -> &mut Vec<(Cow<''static, str>, Variant)>
/**
`attributes` -- Get an immutable reference to the enum's attributes.
*/
pub fn attributes(self: &Self) -> &Attributes
/**
`attributes_mut` -- Get a mutable reference to the enum's attributes.
*/
pub fn attributes_mut(self: &mut Self) -> &mut Attributes
/**
`is_string_enum` -- Check if this enum should be serialized as a string enum.
This is true when all variants are unit variants (no fields).
*/
pub fn is_string_enum(self: &Self) -> bool
```

