# Struct `Deprecated` -- path: `specta::datatype::named::Deprecated`

Runtime representation of Rust's `#[deprecated]` metadata.

```rust
pub struct Deprecated {
	note: Option<std::borrow::Cow<''static, str>>,
	since: Option<std::borrow::Cow<''static, str>>,
}
```

## Methods

```rust
/**
`new` -- Construct deprecation metadata without details.

Eg. `#[deprecated]`
*/
pub fn new() -> Self
/**
`with_note` -- Construct deprecation metadata with a note/message.

Eg. `#[deprecated = "Use something else"]`
*/
pub fn with_note(note: Cow<''static, str>) -> Self
/**
`with_since_note` -- Construct deprecation metadata with a note/message and an optional `since` version.

Eg. `#[deprecated(since = "1.0.0", note = "Use something else")]`
*/
pub fn with_since_note(since: Option<Cow<''static, str>>, note: Cow<''static, str>) -> Self
/**
`note` -- Optional deprecation note/message.
*/
pub fn note(self: &Self) -> Option<&Cow<''static, str>>
/**
`note_mut` -- Mutable optional deprecation note/message.
*/
pub fn note_mut(self: &mut Self) -> Option<&mut Cow<''static, str>>
/**
`set_note` -- Set the optional deprecation note/message.
*/
pub fn set_note(self: &mut Self, note: Option<Cow<''static, str>>)
/**
`since` -- Optional version string from `since = "..."`.
*/
pub fn since(self: &Self) -> Option<&Cow<''static, str>>
/**
`since_mut` -- Mutable optional version string from `since = "..."`.
*/
pub fn since_mut(self: &mut Self) -> Option<&mut Cow<''static, str>>
/**
`set_since` -- Set the optional version string from `since = "..."`.
*/
pub fn set_since(self: &mut Self, since: Option<Cow<''static, str>>)
```

