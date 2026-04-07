# Struct `FloatBits` -- path: `specta::datatype::constant::FloatBits`

Wrapper around f64 bits that implements Eq and Hash.

```rust
pub struct FloatBits (
	u64,
)
```

## Methods

```rust
/**
`from_f64` -- Create from an f64 value.
*/
pub fn from_f64(v: f64) -> Self
/**
`from_f32` -- Create from an f32 value.
*/
pub fn from_f32(v: f32) -> Self
/**
`to_f64` -- Get the f64 value.
*/
pub fn to_f64(self: Self) -> f64
```

