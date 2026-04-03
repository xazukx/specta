# Struct `ResolvedTypes` -- path: `specta::types::ResolvedTypes`

A wrapper around [`Types`] indicating the type graph has already been
transformed for a specific export format.

This is generally constructed by a format crate (for example
[`specta-serde`](https://docs.rs/specta-serde)) after applying
format-specific rewrites.

Constructing this wrapper from plain [`Types`] is explicit because the
conversion may change type shapes. Prefer using your format crate's
conversion entry points when possible.

```rust
pub struct ResolvedTypes (
	Types,
)
```

## Methods

```rust
/**
`from_resolved_types` -- Wrap already-resolved [`Types`] as [`ResolvedTypes`].

This should generally be called by format crates after they finish their
own transformation pass (for example `specta_serde::apply` or
`specta_serde::apply_phases`).

If you call this in end-user code your types may not look how you expect!
*/
pub fn from_resolved_types(types: Types) -> Self
/**
`as_types` -- Borrow the underlying [`Types`] collection.

# Notes

This does not undo format-specific resolution. If a format crate already
rewrote type shapes, this still returns those rewritten shapes. It is your
responsibility to ensure consumers treat these as already-resolved types.
*/
pub fn as_types(self: &Self) -> &Types
/**
`into_types` -- Consume [`ResolvedTypes`] and return the underlying [`Types`].

# Notes

This does not undo format-specific resolution. The returned [`Types`]
remain whatever shape they were resolved into.
*/
pub fn into_types(self: Self) -> Types
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
```

