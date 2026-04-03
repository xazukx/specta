# Struct `Types` -- path: `specta::types::Types`

Define a set of types which can be exported together.

While exporting a type will add all of the types it depends on to the collection.
You can also construct your own collection to easily export a set of types together.

```rust
pub struct Types (
	std::collections::HashMap<reference::NamedId, Option<crate::datatype::NamedDataType>>,
	usize,
)
```

## Methods

```rust
/**
`register` -- Register a [`Type`] with the collection.
*/
pub fn register<T>(self: Self) -> Self
/**
`register_mut` -- Register a [`Type`](crate::Type) with the collection.
*/
pub fn register_mut<T>(self: &mut Self) -> &mut Self
/**
`len` -- Get the length of the collection.
*/
pub fn len(self: &Self) -> usize
/**
`is_empty` -- Check if the collection is empty.
*/
pub fn is_empty(self: &Self) -> bool
/**
`extend` -- Merge types from another collection into this one.
*/
pub fn extend(self: &mut Self, other: &Self)
/**
`into_sorted_iter` -- Sort the collection into a consistent order and return an iterator.

The sort order is not necessarily guaranteed to be stable between versions but currently we sort by name.

This method requires reallocating the map to sort the collection. You should prefer [Self::into_unsorted_iter] if you don't care about the order.
*/
pub fn into_sorted_iter(self: &Self) -> impl 
/**
`into_unsorted_iter` -- Return the unsorted iterator over the collection.
*/
pub fn into_unsorted_iter(self: &Self) -> impl 
/**
`iter_mut` -- Return an mutable iterator over the type collection.
Note: The order returned is unsorted.
*/
pub fn iter_mut<F>(self: &mut Self, f: F)
/**
`map` -- Map over the collection, transforming each `NamedDataType` with the given closure.
This preserves the `ArcId` keys, ensuring that `Reference`s remain valid.
*/
pub fn map<F>(self: Self, f: F) -> Self
```

