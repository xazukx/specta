# Functions in specta::export::topo_sort

```rust
/**
`topological_sort_types` -- Topologically sort types so dependencies are emitted before the types that reference them.
*/
pub fn topological_sort_types<'a>(ndts: Vec<&crate::datatype::NamedDataType>, types: &crate::Types) -> Vec<&crate::datatype::NamedDataType>
```

