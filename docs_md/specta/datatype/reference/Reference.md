# Enum `Reference` -- path: `specta::datatype::reference::Reference`

Reference to another type.

```rust
pub enum Reference {
	/// Reference to a named type collected in a [`Types`].
	/// [NamedReference](./NamedReference.md)
	Named(NamedReference),
	/// Reference to a generic type parameter.
	/// [GenericReference](./GenericReference.md)
	Generic(GenericReference),
	/// Reference to an opaque exporter-specific type.
	/// [OpaqueReference](./OpaqueReference.md)
	Opaque(OpaqueReference),
}
```

## Methods

```rust
/**
`opaque` -- Construct a new reference to an opaque type.

An opaque type is unable to be represented using the [DataType] system and requires specific exporter integration to handle it.

Opaque [Reference]'s are compared using [PartialEq]. For example `Reference::opaque(()) == Reference::opaque(())` so you must ensure each reference you intent to be unique is implemented as such.
*/
pub fn opaque<T>(state: T) -> Self
/**
`ty_eq` -- Compare if two references point to the same type.

This is different from using `Eq`, `PartialEq`, or `Hash` as those compare the [Reference].
A [Reference] contains generics, inline and other attributes which this ignores.
*/
pub fn ty_eq(self: &Self, other: &Reference) -> bool
/**
`inline` -- Convert an existing [Reference] into an inlined one.

It's not safe to go the other way incase the type is inlined which requires all [Reference]'s to be inlined.
*/
pub fn inline(self: Self) -> Reference
```

