# Enum `EnumRepr` -- path: `specta_serde::repr::EnumRepr`

Serde representation of an enum.
Refer to the [Serde documentation](https://serde.rs/enum-representations.html) for more information.

```rust
pub enum EnumRepr {
	/// Untagged enum representation.
	Untagged,
	/// Externally tagged enum representation.
	External,
	/// Internally tagged enum representation.
	Internal { tag: std::borrow::Cow<''static, str> },
	/// Adjacently tagged enum representation.
	Adjacent { tag: std::borrow::Cow<''static, str>, content: std::borrow::Cow<''static, str> },
}
```
