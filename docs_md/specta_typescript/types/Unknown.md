# Struct `Unknown` -- path: `specta_typescript::types::Unknown`

Cast a Rust type to a Typescript `unknown` type.

# Examples

This can be used as a type override.
```rust
use serde::Serialize;
use specta::Type;
use specta_typescript::Unknown;

#[derive(Serialize, Type)]
pub struct Demo {
    #[specta(type = Unknown)]
    pub field: String,
}
```

Or it can be used as a wrapper type.
```rust
use serde::Serialize;
use specta::Type;
use specta_typescript::Unknown;

# #[cfg(feature = "serde")] {
#[derive(Serialize, Type)]
pub struct Demo {
    pub field: Unknown<String>,
}
# }
```

```rust
pub struct Unknown (
	T,
)
```
