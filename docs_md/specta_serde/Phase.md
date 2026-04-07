# Enum `Phase` -- path: `specta_serde::Phase`

Selects which directional type shape to use after [`apply_phases`].

```rust
pub enum Phase {
	/// The shape used when Rust serializes data to the wire.
	Serialize,
	/// The shape used when Rust deserializes data from the wire.
	Deserialize,
}
```
