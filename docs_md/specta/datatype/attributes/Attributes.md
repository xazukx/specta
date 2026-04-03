# Struct `Attributes` -- path: `specta::datatype::attributes::Attributes`

A named map of type-erased metadata attached to datatype nodes.

`Attributes` is primarily used by advanced consumers that need to inspect
metadata recorded on [`DataType`](super::DataType) nodes at runtime. Each
entry is stored under a string key and can later be retrieved either as a
raw [`Any`] value or by downcasting to the original type.

Stored values must be owned and implement [`Clone`], [`Eq`], [`Hash`], and
[`fmt::Debug`] so attributes remain comparable, hashable, and printable as
part of the surrounding datatype graph.

# Examples

```rust
use specta::datatype::Attributes;

let mut attrs = Attributes::default();
attrs.insert("serde.rename", String::from("user_name"));
attrs.insert("serde.skip", true);

assert_eq!(attrs.len(), 2);
assert!(attrs.contains_key("serde.rename"));
assert_eq!(
    attrs.get_named_as::<String>("serde.rename"),
    Some(&String::from("user_name"))
);
assert_eq!(attrs.get_named_as::<bool>("serde.skip"), Some(&true));
assert_eq!(attrs.get_named_as::<u32>("serde.skip"), None);
```

```rust
pub struct Attributes (
	std::collections::HashMap<std::borrow::Cow<''static, str>, std::sync::Arc<dyn [DynAttributeValue](./DynAttributeValue.md)>>,
)
```

## Methods

```rust
/**
`len` -- Returns the number of stored attribute entries.
*/
pub fn len(self: &Self) -> usize
/**
`is_empty` -- Returns `true` when the collection has no entries.
*/
pub fn is_empty(self: &Self) -> bool
/**
`insert` -- Inserts or replaces an attribute value.

Values are stored in a type-erased form, but they must still implement
[`Clone`], [`Eq`], [`Hash`], and [`fmt::Debug`] so the containing
[`Attributes`] remains cloneable, comparable, hashable, and printable.

# Examples

```rust
use specta::datatype::Attributes;

let mut attrs = Attributes::default();
attrs.insert("serde.default", true);

assert_eq!(attrs.get_named_as::<bool>("serde.default"), Some(&true));
```
*/
pub fn insert<T, impl Into<Cow<'static, str>>>(self: &mut Self, key: impl , value: T)
/**
`extend` -- Extends `self` with entries from `other`.

If both collections contain the same key, the value from `other`
replaces the existing entry in `self`.

# Examples

```rust
use specta::datatype::Attributes;

let mut base = Attributes::default();
base.insert("serde.rename", String::from("first_name"));

let mut extra = Attributes::default();
extra.insert("serde.skip", true);

base.extend(extra);

assert_eq!(base.get_named_as::<bool>("serde.skip"), Some(&true));
```
*/
pub fn extend(self: &mut Self, other: Self)
/**
`contains_key` -- Returns `true` if an attribute entry is present for `key`.
*/
pub fn contains_key(self: &Self, key: &str) -> bool
/**
`get_named` -- Returns the raw type-erased value for a named attribute.

This is useful when the expected type is not known until runtime.
Prefer [`Attributes::get_named_as`] when you know the concrete type.

# Examples

```rust
use specta::datatype::Attributes;

let mut attrs = Attributes::default();
attrs.insert("serde.rename", String::from("user_name"));

let value = attrs.get_named("serde.rename").unwrap();
assert_eq!(value.downcast_ref::<String>(), Some(&String::from("user_name")));
```
*/
pub fn get_named(self: &Self, key: &str) -> Option<&dyn Any>
/**
`get_named_as` -- Returns a typed reference to the named attribute value.

Returns `None` when the key is missing or when the stored value has a
different type than `T`.

# Examples

```rust
use specta::datatype::Attributes;

let mut attrs = Attributes::default();
attrs.insert("serde.skip", true);

assert_eq!(attrs.get_named_as::<bool>("serde.skip"), Some(&true));
assert_eq!(attrs.get_named_as::<String>("serde.skip"), None);
```
*/
pub fn get_named_as<T>(self: &Self, key: &str) -> Option<&T>
```

