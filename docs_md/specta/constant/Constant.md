# Trait `Constant` -- path: `specta::constant::Constant`

Trait for types that represent a compile-time constant exportable to TypeScript.

- auto: no | unsafe: no | dyn-compatible: no


## Methods

```rust
/**
`name` -- The name to use in the TypeScript export.
*/
fn name() -> Cow<''static, str>
/**
`value` -- The resolved constant value.
*/
fn value() -> ConstantValue
/**
`docs` -- Documentation comments on this constant.
*/
fn docs() -> Cow<''static, str>
/**
`deprecated` -- Whether this constant is deprecated.
*/
fn deprecated() -> Option<crate::datatype::Deprecated>
/**
`module_path` -- The Rust module path where this constant is defined.
*/
fn module_path() -> Cow<''static, str>
/**
`register` -- Register this constant into a [`Constants`] collection.
*/
fn register(constants: &mut Constants)
```

## Implemented by

_No known implementors in this crate._

