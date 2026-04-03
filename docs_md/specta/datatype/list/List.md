# Struct `List` -- path: `specta::datatype::list::List`

List of items. This will be a [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html) or similar types.

```rust
pub struct List {
	ty: Box<super::DataType>,
	length: Option<usize>,
	unique: bool,
}
```

## Methods

```rust
/**
`new` -- Create a new list of a given type.
*/
pub fn new(ty: DataType) -> Self
/**
`ty` -- Get an immutable reference to the type of the elements in the list.
*/
pub fn ty(self: &Self) -> &DataType
/**
`ty_mut` -- Get a mutable reference to the type of the elements in the list.
*/
pub fn ty_mut(self: &mut Self) -> &mut DataType
/**
`set_ty` -- Set the type of the elements in the list.
*/
pub fn set_ty(self: &mut Self, ty: DataType)
/**
`length` -- Get the length of the list.

Length is set for `[Type; N]` arrays.
*/
pub fn length(self: &Self) -> Option<usize>
/**
`set_length` -- Set the length of the list.

Length is set for `[Type; N]` arrays.
*/
pub fn set_length(self: &mut Self, length: Option<usize>)
/**
`unique` -- Are each elements unique? Eg. `HashSet` or `BTreeSet`
*/
pub fn unique(self: &Self) -> bool
/**
`set_unique` -- Set whether each element is unique.
*/
pub fn set_unique(self: &mut Self, unique: bool)
```

