# Struct `Field` -- path: `specta::datatype::fields::Field`

Field metadata for a struct field or enum variant field.

```rust
pub struct Field {
	/// Did the user apply a `#[specta(optional)]` attribute.
	optional: bool,
	/// Did the user apply a `#[serde(flatten)]` attribute.
	flatten: bool,
	/// Deprecated attribute for the field.
	deprecated: Option<super::Deprecated>,
	/// Documentation comments for the field.
	docs: std::borrow::Cow<''static, str>,
	/// Should we inline the definition of this type.
	inline: bool,
	/// Did the user apply a `#[specta(type = ...)]` or `#[specta(r#type = ...)]` attribute.
	type_overridden: bool,
	/// Runtime attributes for this field.
	attributes: super::Attributes,
	/// Type for the field. Is optional if `#[serde(skip)]` or `#[specta(skip)]` was applied.
	/// 
	/// You might think, well why not apply this in the macro and just not emit the variant?
	/// Well in Serde `A(String)` and `A(#[serde(skip)] (), String)` export as different Typescript types so the exporter needs runtime knowledge of this.
	ty: Option<super::DataType>,
}
```

## Methods

```rust
/**
`new` -- Construct a new field with the given type.

You can skip the requirement on providing a [`DataType`] by using [`Field::default`]
*/
pub fn new(ty: DataType) -> Self
/**
`optional` -- Has the Serde or Specta optional attribute been applied to this field?
*/
pub fn optional(self: &Self) -> bool
/**
`set_optional` -- Set the optional attribute for this field.
*/
pub fn set_optional(self: &mut Self, optional: bool)
/**
`flatten` -- Has the Serde flatten attribute been applied to this field?
*/
pub fn flatten(self: &Self) -> bool
/**
`set_flatten` -- Set the flatten attribute for this field.
*/
pub fn set_flatten(self: &mut Self, flatten: bool)
/**
`inline` -- Has the Serde inline attribute been applied to this field?
*/
pub fn inline(self: &Self) -> bool
/**
`set_inline` -- Set the inline attribute for this field.
*/
pub fn set_inline(self: &mut Self, inline: bool)
/**
`type_overridden` -- Has the Specta type override attribute been applied to this field?
*/
pub fn type_overridden(self: &Self) -> bool
/**
`set_type_overridden` -- Set whether a Specta type override attribute was applied to this field.
*/
pub fn set_type_overridden(self: &mut Self, type_overridden: bool)
/**
`deprecated` -- Has the Rust deprecated attribute been applied to this field?
*/
pub fn deprecated(self: &Self) -> Option<&Deprecated>
/**
`deprecated_mut` -- Has the Rust deprecated attribute been applied to this field?
*/
pub fn deprecated_mut(self: &mut Self) -> Option<&mut Deprecated>
/**
`set_deprecated` -- Set the deprecated attribute for this field.
*/
pub fn set_deprecated(self: &mut Self, deprecated: Option<Deprecated>)
/**
`docs` -- Get an immutable reference to the documentation attribute for this field.
*/
pub fn docs(self: &Self) -> &Cow<''static, str>
/**
`docs_mut` -- Mutable reference to the documentation attribute for this field.
*/
pub fn docs_mut(self: &mut Self) -> &mut Cow<''static, str>
/**
`set_docs` -- Set the documentation attribute for this field.
*/
pub fn set_docs(self: &mut Self, docs: Cow<''static, str>)
/**
`ty` -- Get an immutable reference to the type of this field.
*/
pub fn ty(self: &Self) -> Option<&DataType>
/**
`ty_mut` -- Mutable reference to the type of this field.
*/
pub fn ty_mut(self: &mut Self) -> Option<&mut DataType>
/**
`set_ty` -- Set the type of this field.
*/
pub fn set_ty(self: &mut Self, ty: DataType)
/**
`attributes` -- Get an immutable reference to the runtime attributes for this field.
*/
pub fn attributes(self: &Self) -> &Attributes
/**
`attributes_mut` -- Mutable reference to the runtime attributes for this field.
*/
pub fn attributes_mut(self: &mut Self) -> &mut Attributes
/**
`set_attributes` -- Set the runtime attributes for this field.
*/
pub fn set_attributes(self: &mut Self, attrs: Attributes)
```

