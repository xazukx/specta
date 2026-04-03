# Struct `GenericReference` -- path: `specta::datatype::reference::GenericReference`

Reference to a generic parameter a parent [NamedDataType].
This is resolved to a concrete type by the language exporter.

```rust
pub struct GenericReference {
	id: std::any::TypeId,
}
```

## Methods

```rust
/**
`new` -- Build a new [GenericReference] for a generic type parameter marker.
`T` should be a unique type which identifies the generic (Eg. `pub struct GenericT;`) and must be registered on the parent [`NamedDataType`].
*/
pub fn new<T>() -> Self
/**
`eq` -- Compare two [GenericReference]s for equality.
If this returns true they are both a reference to the same generic type.
*/
pub fn eq<T>(self: &Self) -> bool
```

