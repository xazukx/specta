# Struct `Constants` -- path: `specta::constants::Constants`

A collection of constants to export, analogous to [`crate::Types`] for type declarations.

# Examples

```ignore
use specta::Constants;

let constants = Constants::default()
    .register::<MyApiVersion>()
    .register::<MaxRetries>();
```

```rust
pub struct Constants (
	Vec<crate::constant::NamedConstant>,
)
```

## Methods

```rust
/**
`register` -- Register a [`Constant`] with the collection.
*/
pub fn register<C>(self: Self) -> Self
/**
`register_mut` -- Register a [`Constant`] with the collection by mutable reference.
*/
pub fn register_mut<C>(self: &mut Self) -> &mut Self
/**
`push` -- Push a named constant directly.
*/
pub fn push(self: &mut Self, constant: NamedConstant)
/**
`len` -- Get the number of constants in the collection.
*/
pub fn len(self: &Self) -> usize
/**
`is_empty` -- Check if the collection is empty.
*/
pub fn is_empty(self: &Self) -> bool
/**
`into_sorted_iter` -- Return a sorted iterator over the constants (sorted by name).
*/
pub fn into_sorted_iter(self: &Self) -> impl 
/**
`iter` -- Return an unsorted iterator over the constants.
*/
pub fn iter(self: &Self) -> impl 
```

