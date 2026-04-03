# Struct `Never` -- path: `specta_typescript::types::Never`

Cast a Rust type to a Typescript `never` type.

# Examples

This can be used as a type override.
```rust
use serde::Serialize;
use specta::Type;
use specta_typescript::Never;

#[derive(Serialize, Type)]
pub struct Demo {
    #[specta(type = Never)]
    pub field: String,
}
```

Or it can be used as a wrapper type.
```rust
use serde::Serialize;
use specta::Type;
use specta_typescript::Never;

# #[cfg(feature = "serde")] {
#[derive(Serialize, Type)]
pub struct Demo {
    pub field: Never<String>,
}
# }
```

```rust
pub struct Never (
	T,
)
```
