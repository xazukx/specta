# Struct `Function` -- path: `specta::datatype::function::Function`

Contains type information about a function annotated with the `#[specta]` attribute.
Returned by the `fn_datatype!` macro.

```rust
pub struct Function {
	/// Whether the function is async.
	asyncness: bool,
	/// The function's name.
	name: std::borrow::Cow<''static, str>,
	/// The name and type of each of the function's arguments.
	args: Vec<(std::borrow::Cow<''static, str>, super::DataType)>,
	/// The return type of the function.
	result: Option<super::DataType>,
	/// The function's documentation. Detects both `///` and `#[doc = ...]` style documentation.
	docs: std::borrow::Cow<''static, str>,
	/// The deprecated status of the function.
	deprecated: Option<super::Deprecated>,
}
```

## Methods

```rust
/**
`asyncness` -- Is this function defined with the `async` keyword?
*/
pub fn asyncness(self: &Self) -> bool
/**
`set_asyncness` -- Set the `async` status of the function.
*/
pub fn set_asyncness(self: &mut Self, asyncness: bool)
/**
`name` -- Get the name of the function.
*/
pub fn name(self: &Self) -> &Cow<''static, str>
/**
`name_mut` -- Get a mutable reference to the name of the function.
*/
pub fn name_mut(self: &mut Self) -> &mut Cow<''static, str>
/**
`set_name` -- Set the name of the function.
*/
pub fn set_name(self: &mut Self, name: Cow<''static, str>)
/**
`args` -- Get the arguments of the function.
*/
pub fn args(self: &Self) -> &[(Cow<''static, str>, DataType)]
/**
`args_mut` -- Get the arguments of the function as mutable references.
*/
pub fn args_mut(self: &mut Self) -> &mut Vec<(Cow<''static, str>, DataType)>
/**
`result` -- Get the result of the function.
*/
pub fn result(self: &Self) -> Option<&DataType>
/**
`result_mut` -- Get the result of the function as mutable reference.
*/
pub fn result_mut(self: &mut Self) -> Option<&mut DataType>
/**
`set_result` -- Set the result of the function.
*/
pub fn set_result(self: &mut Self, result: DataType)
/**
`docs` -- Get the documentation of the function.
*/
pub fn docs(self: &Self) -> &Cow<''static, str>
/**
`docs_mut` -- Get the documentation of the function as mutable reference.
*/
pub fn docs_mut(self: &mut Self) -> &mut Cow<''static, str>
/**
`set_docs` -- Set the documentation of the function.
*/
pub fn set_docs(self: &mut Self, docs: Cow<''static, str>)
/**
`deprecated` -- Get the deprecated status of the function.
*/
pub fn deprecated(self: &Self) -> Option<&Deprecated>
/**
`deprecated_mut` -- Get the deprecated status of the function as mutable reference.
*/
pub fn deprecated_mut(self: &mut Self) -> Option<&mut Deprecated>
/**
`set_deprecated` -- Set the deprecated status of the function.
*/
pub fn set_deprecated(self: &mut Self, deprecated: Deprecated)
```

