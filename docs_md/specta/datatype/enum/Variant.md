# Struct `Variant` -- path: `specta::datatype::enum::Variant`

represents a variant of an enum.

```rust
pub struct Variant {
	/// Did the user apply a `#[serde(skip)]` or `#[specta(skip)]` attribute.
	/// 
	/// You might think, well why not apply this in the macro and just not emit the variant?
	/// Well in Serde `A(String)` and `A(#[serde(skip)] (), String)` export as different Typescript types so the exporter needs runtime knowledge of this.
	skip: bool,
	/// Documentation comments for the field.
	docs: std::borrow::Cow<''static, str>,
	/// Deprecated attribute for the field.
	deprecated: Option<super::Deprecated>,
	/// The type of the variant.
	fields: super::Fields,
	/// Runtime attributes for this variant
	attributes: super::Attributes,
	/// Did the user apply a `#[specta(type = ...)]` or `#[specta(r#type = ...)]` attribute.
	type_overridden: bool,
}
```

## Methods

```rust
/**
`unit` -- Construct a new unit enum variant.
*/
pub fn unit() -> Self
/**
`named` -- Construct a new struct enum variant with named fields.
*/
pub fn named() -> VariantBuilder<NamedFields>
/**
`unnamed` -- Construct a new tuple enum variant without unnamed fields.
*/
pub fn unnamed() -> VariantBuilder<UnnamedFields>
/**
`skip` -- Has the Serde or Specta skip attribute been applied to this variant?
*/
pub fn skip(self: &Self) -> bool
/**
`set_skip` -- Set the skip attribute for the variant.
*/
pub fn set_skip(self: &mut Self, skip: bool)
/**
`docs` -- Get an immutable reference to the documentation comments for the field.
*/
pub fn docs(self: &Self) -> &Cow<''static, str>
/**
`docs_mut` -- Get a mutable reference to the documentation comments for the variant.
*/
pub fn docs_mut(self: &mut Self) -> &mut Cow<''static, str>
/**
`set_docs` -- Set the documentation comments for the field.
*/
pub fn set_docs(self: &mut Self, docs: Cow<''static, str>)
/**
`deprecated` -- Get an immutable reference to the deprecated attribute for the field.
*/
pub fn deprecated(self: &Self) -> Option<&Deprecated>
/**
`deprecated_mut` -- Get a mutable reference to the deprecated attribute for the field.
*/
pub fn deprecated_mut(self: &mut Self) -> Option<&mut Deprecated>
/**
`set_deprecated` -- Set the deprecated attribute for the field.
*/
pub fn set_deprecated(self: &mut Self, deprecated: Option<Deprecated>)
/**
`fields` -- Get an immutable reference to the fields of the variant.
*/
pub fn fields(self: &Self) -> &Fields
/**
`fields_mut` -- Get a mutable reference to the fields of the variant.
*/
pub fn fields_mut(self: &mut Self) -> &mut Fields
/**
`attributes` -- Get an immutable reference to the runtime attributes for this variant.
*/
pub fn attributes(self: &Self) -> &Attributes
/**
`attributes_mut` -- Mutable reference to the runtime attributes for this variant.
*/
pub fn attributes_mut(self: &mut Self) -> &mut Attributes
/**
`type_overridden` -- Has the Specta type override attribute been applied to this variant?
*/
pub fn type_overridden(self: &Self) -> bool
/**
`set_type_overridden` -- Set whether a Specta type override attribute was applied to this variant.
*/
pub fn set_type_overridden(self: &mut Self, type_overridden: bool)
```

