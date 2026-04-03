# Struct `NamedDataType` -- path: `specta::datatype::named::NamedDataType`

Named type represents any type with it's own unique name and identity.

These can become `export MyNamedType = ...` in Typescript can we be referenced in types like `{ field: MyNamedType }`.

```rust
pub struct NamedDataType {
	id: crate::datatype::reference::NamedId,
	name: std::borrow::Cow<''static, str>,
	docs: std::borrow::Cow<''static, str>,
	deprecated: Option<Deprecated>,
	module_path: std::borrow::Cow<''static, str>,
	location: std::panic::Location<''static>,
	generics: std::borrow::Cow<''static, [(crate::datatype::reference::GenericReference, std::borrow::Cow<''static, str>)]>,
	inline: bool,
	inner: crate::datatype::DataType,
}
```

## Methods

```rust
/**
`new` -- Construct a new named datatype.

Note: Ensure you call `Self::register` to register the type.
*/
pub fn new<impl Into<Cow<'static, str>>>(name: impl , generics: Vec<(GenericReference, Cow<''static, str>)>, dt: DataType) -> Self
/**
`new_inline` -- Construct a new inlined named datatype.

Note: Ensure you call `Self::register` to register the type.
*/
pub fn new_inline<impl Into<Cow<'static, str>>>(name: impl , generics: Vec<(GenericReference, Cow<''static, str>)>, dt: DataType) -> Self
/**
`register` -- Register the type into a [Types].
*/
pub fn register(self: &Self, types: &mut Types)
/**
`reference` -- Construct a [Reference] to a [NamedDataType].
This can be included in a `DataType::Reference` within another type.

This reference will be inlined if the type is inlined, otherwise you can inline it with [Reference::inline].
*/
pub fn reference(self: &Self, generics: Vec<(GenericReference, DataType)>) -> Reference
/**
`requires_reference` -- Check whether a type requires a reference to be generated.

This if `false` is all [Reference]'s created for the type are inlined,
in that case it doesn't need to be exported because it will never be
referenced.
*/
pub fn requires_reference(self: &Self, _types: &Types) -> bool
/**
`name` -- The name of the type
*/
pub fn name(self: &Self) -> &Cow<''static, str>
/**
`name_mut` -- Get a mutable reference to the name of the type
*/
pub fn name_mut(self: &mut Self) -> &mut Cow<''static, str>
/**
`set_name` -- Set the name of the type
*/
pub fn set_name(self: &mut Self, name: Cow<''static, str>)
/**
`docs` -- Rust documentation comments on the type
*/
pub fn docs(self: &Self) -> &Cow<''static, str>
/**
`docs_mut` -- Get a mutable reference to the Rust documentation comments on the type
*/
pub fn docs_mut(self: &mut Self) -> &mut Cow<''static, str>
/**
`set_docs` -- Set the Rust documentation comments on the type
*/
pub fn set_docs(self: &mut Self, docs: Cow<''static, str>)
/**
`deprecated` -- The Rust deprecated comment if the type is deprecated.
*/
pub fn deprecated(self: &Self) -> Option<&Deprecated>
/**
`deprecated_mut` -- Get a mutable reference to the Rust deprecated comment if the type is deprecated.
*/
pub fn deprecated_mut(self: &mut Self) -> Option<&mut Deprecated>
/**
`set_deprecated` -- Set the Rust deprecated comment if the type is deprecated.
*/
pub fn set_deprecated(self: &mut Self, deprecated: Option<Deprecated>)
/**
`location` -- The code location where this type is implemented
*/
pub fn location(self: &Self) -> Location<''static>
/**
`set_location` -- Set the code location where this type is implemented
*/
pub fn set_location(self: &mut Self, location: Location<''static>)
/**
`module_path` -- The Rust path of the module where this type is defined
*/
pub fn module_path(self: &Self) -> &Cow<''static, str>
/**
`module_path_mut` -- Get a mutable reference to the Rust path of the module where this type is defined
*/
pub fn module_path_mut(self: &mut Self) -> &mut Cow<''static, str>
/**
`set_module_path` -- Set the Rust path of the module where this type is defined
*/
pub fn set_module_path(self: &mut Self, module_path: Cow<''static, str>)
/**
`generics` -- The generics that are defined on this type
*/
pub fn generics(self: &Self) -> &[(GenericReference, Cow<''static, str>)]
/**
`generics_mut` -- Get a mutable reference to the generics that are defined on this type
*/
pub fn generics_mut(self: &mut Self) -> &mut Cow<''static, [(GenericReference, Cow<''static, str>)]>
/**
`ty` -- Get the inner [`DataType`]
*/
pub fn ty(self: &Self) -> &DataType
/**
`ty_mut` -- Get a mutable reference to the inner [`DataType`]
*/
pub fn ty_mut(self: &mut Self) -> &mut DataType
/**
`set_ty` -- Set the inner [`DataType`]
*/
pub fn set_ty(self: &mut Self, ty: DataType)
```

