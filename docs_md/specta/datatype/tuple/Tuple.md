# Struct `Tuple` -- path: `specta::datatype::tuple::Tuple`

Represents a Rust [tuple](https://doc.rust-lang.org/std/primitive.tuple.html) type.

Be aware `()` is treated specially as `null` when using the Typescript exporter.

```rust
pub struct Tuple {
	elements: Vec<super::DataType>,
}
```

## Methods

```rust
/**
`new` -- Create a new tuple with the given elements.
*/
pub fn new(elements: Vec<DataType>) -> Self
/**
`elements` -- Get the elements of the tuple.
*/
pub fn elements(self: &Self) -> &[DataType]
/**
`elements_mut` -- Get a mutable reference to the elements of the tuple.
*/
pub fn elements_mut(self: &mut Self) -> &mut Vec<DataType>
```

