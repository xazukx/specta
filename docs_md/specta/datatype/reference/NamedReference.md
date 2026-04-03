# Struct `NamedReference` -- path: `specta::datatype::reference::NamedReference`

Reference to a [NamedDataType].

```rust
pub struct NamedReference {
	id: NamedId,
	generics: Vec<(GenericReference, super::DataType)>,
	inline: bool,
}
```

## Methods

```rust
/**
`get` -- Get a reference to a [NamedDataType] from a [Types].

This is guaranteed to return a [NamedDataType] if the [Types] matches,
what was used to get the original [Reference].
*/
pub fn get<'a>(self: &Self, types: &Types) -> Option<&NamedDataType>
/**
`generics` -- Get the generic parameters set on this reference which will be filled in by the [NamedDataType].
*/
pub fn generics(self: &Self) -> &[(GenericReference, DataType)]
/**
`generics_mut` -- Get the generic parameters set on this reference as mutable references.
*/
pub fn generics_mut(self: &mut Self) -> &mut Vec<(GenericReference, DataType)>
/**
`inline` -- Get whether this reference should be inlined
*/
pub fn inline(self: &Self) -> bool
```

