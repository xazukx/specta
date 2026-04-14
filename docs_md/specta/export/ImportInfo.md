# Struct `ImportInfo` -- path: `specta::export::ImportInfo`

Information about what is imported from a specific module.

```rust
pub struct ImportInfo {
	/// The specific type names imported from this module.
	pub type_names: std::collections::BTreeSet<String>,
	/// Whether any imported item is a runtime value (not just a type).
	/// Relevant for TypeScript `import` vs `import type` distinction.
	pub has_values: bool,
}
```

## Methods

```rust
/**
`new` -- Create a new empty ImportInfo.
*/
pub fn new() -> Self
/**
`merge` -- Merge another ImportInfo into this one.
*/
pub fn merge(self: &mut Self, other: &ImportInfo)
```

