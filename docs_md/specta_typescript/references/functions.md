# Functions in specta_typescript::references

```rust
/**
`collect_references` -- This function collects all Typescript references which are created within the given closure.

This can be used for determining the imports required in a particular file.
*/
pub fn collect_references<R, impl FnOnce() -> R>(func: impl ) -> (R, std::collections::HashSet<specta::datatype::NamedReference>)
```

