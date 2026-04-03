# Struct `Any` -- path: `specta_typescript::types::Any`

Cast a Rust type to a Typescript `any` type.

WARNING: When used with `Option<Any<T>>`, Typescript will not prompt you about nullability checks as `any | null` is coalesced to `any` in Typescript.

# Examples

This can be used as a type override.
```rust
use serde::Serialize;
use specta::Type;
use specta_typescript::Any;

#[derive(Serialize, Type)]
pub struct Demo {
    #[specta(type = Any)]
    pub field: String,
}
```

Or it can be used as a wrapper type.
```rust
use serde::Serialize;
use specta::Type;
use specta_typescript::Any;

# #[cfg(feature = "serde")] {
#[derive(Serialize, Type)]
pub struct Demo {
    pub field: Any<String>,
}
# }
```

```rust
pub struct Any (
	T,
)
```
