# Struct `OpaqueReference` -- path: `specta::datatype::reference::OpaqueReference`

Reference to a type not understood by Specta's core.

These are implemented by the language exporter to implement cool features like
[`specta_typescript::branded!`](https://docs.rs/specta-typescript/latest/specta_typescript/macro.branded.html),
[`specta_typescript::define`](https://docs.rs/specta-typescript/latest/specta_typescript/fn.define.html), and more.

This is an advanced feature designed for language exporters so should generally be avoided and is not intended to be generally useful unless your in control of the language exporter.

```rust
pub struct OpaqueReference (
	std::sync::Arc<dyn [DynOpaqueReference](./DynOpaqueReference.md)>,
)
```

## Methods

```rust
/**
`type_name` -- Get the Rust type name of the stored opaque state.
*/
pub fn type_name(self: &Self) -> &str
/**
`type_id` -- Get the [`TypeId`] of the stored opaque state.
*/
pub fn type_id(self: &Self) -> TypeId
/**
`downcast_ref` -- Attempt to downcast the opaque state to `T`.
*/
pub fn downcast_ref<T>(self: &Self) -> Option<&T>
```

