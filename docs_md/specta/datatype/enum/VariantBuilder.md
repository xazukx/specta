# Struct `VariantBuilder` -- path: `specta::datatype::enum::VariantBuilder`

Builder for constructing [`Variant`] values.

```rust
pub struct VariantBuilder {
	v: Variant,
	variant: V,
}
```

## Methods

```rust
/**
`skip` -- Mark the variant as skipped.
*/
pub fn skip(self: Self) -> Self
/**
`docs` -- Set documentation for the variant.
*/
pub fn docs(self: Self, docs: Cow<''static, str>) -> Self
/**
`deprecated` -- Set deprecation metadata for the variant.
*/
pub fn deprecated(self: Self, reason: Deprecated) -> Self
/**
`attributes` -- Set runtime attributes on the variant.
*/
pub fn attributes(self: Self, attributes: Attributes) -> Self
/**
`attributes_mut` -- Set runtime attributes on the variant in-place.
*/
pub fn attributes_mut(self: &mut Self, attributes: Attributes)
/**
`type_overridden` -- Set whether the variant has a Specta type override.
*/
pub fn type_overridden(self: Self, type_overridden: bool) -> Self
/**
`type_overridden_mut` -- Set whether the variant has a Specta type override in-place.
*/
pub fn type_overridden_mut(self: &mut Self, type_overridden: bool)
/**
`field` -- Add an unnamed field to the variant.
*/
pub fn field(self: Self, field: Field) -> Self
/**
`field_mut` -- Add an unnamed field to the variant and return the updated builder.
*/
pub fn field_mut(self: Self, field: Field) -> Self
/**
`build` -- Finalize unnamed variant builder into [Variant].
*/
pub fn build(self: Self) -> Variant
/**
`field` -- Add a named field to the variant.
*/
pub fn field<impl Into<Cow<'static, str>>>(self: Self, name: impl , field: Field) -> Self
/**
`field_mut` -- Add a named field to the variant and return the updated builder.
*/
pub fn field_mut<impl Into<Cow<'static, str>>>(self: Self, name: impl , field: Field) -> Self
/**
`build` -- Finalize named variant builder into [Variant].
*/
pub fn build(self: Self) -> Variant
```

