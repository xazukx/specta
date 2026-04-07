# Struct `Phased` -- path: `specta_serde::phased::Phased`

Declares an explicit serialize/deserialize type pair for Specta output.

This is primarily used with `#[specta(type = ...)]` when serde attributes
cause the wire shape to differ by direction.

- `Serialize` is the type used for serialization output.
- `Deserialize` is the type accepted for deserialization input.

When both phases resolve to the same Specta datatype, this collapses to that
single type. When they differ, `apply_phases` can split the graph into
`*_Serialize` and `*_Deserialize` variants.

```rust
# use specta::Type;
#[derive(Type)]
struct OneOrMany {
    #[specta(type = specta_serde::Phased<Vec<String>, String>)]
    value: Vec<String>,
}
```

```rust
pub struct Phased (
	std::marker::PhantomData<(Serialize, Deserialize)>,
)
```
