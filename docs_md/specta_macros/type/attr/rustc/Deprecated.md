# Struct `Deprecated` -- path: `specta_macros::type::attr::rustc::Deprecated`

```rust
pub struct Deprecated {
	note: Option<std::borrow::Cow<''static, str>>,
	since: Option<std::borrow::Cow<''static, str>>,
}
```

## Methods

```rust

pub fn new() -> Self

pub fn with_note(note: Cow<''static, str>) -> Self

pub fn with_since_note(since: Option<Cow<''static, str>>, note: Cow<''static, str>) -> Self

pub fn note(self: &Self) -> Option<&Cow<''static, str>>

pub fn since(self: &Self) -> Option<&Cow<''static, str>>
```

