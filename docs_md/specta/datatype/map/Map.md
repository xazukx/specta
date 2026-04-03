# Struct `Map` -- path: `specta::datatype::map::Map`

Map of items. This will be a [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) or similar types.

```rust
pub struct Map (
	Box<(super::DataType, super::DataType)>,
)
```

## Methods

```rust
/**
`new` -- Create a new map with the given key and value types.
*/
pub fn new(key_ty: DataType, value_ty: DataType) -> Self
/**
`key_ty` -- The type of the map keys.
*/
pub fn key_ty(self: &Self) -> &DataType
/**
`key_ty_mut` -- Get a mutable reference to the type of the map keys.
*/
pub fn key_ty_mut(self: &mut Self) -> &mut DataType
/**
`set_key_ty` -- Set the type of the map keys.
*/
pub fn set_key_ty(self: &mut Self, key_ty: DataType)
/**
`value_ty` -- The type of the map values.
*/
pub fn value_ty(self: &Self) -> &DataType
/**
`value_ty_mut` -- Get a mutable reference to the type of the map values.
*/
pub fn value_ty_mut(self: &mut Self) -> &mut DataType
/**
`set_value_ty` -- Set the type of the map values.
*/
pub fn set_value_ty(self: &mut Self, value_ty: DataType)
```

