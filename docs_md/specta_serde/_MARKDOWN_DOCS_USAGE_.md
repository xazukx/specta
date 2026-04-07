# Documentation Summary

Crate: `specta_serde`
Version: `0.0.11`

## How to use these docs

This directory contains Markdown documentation generated from rustdoc JSON.
The files are organized to mirror the crate's module structure:

- Each struct, enum, and trait has its own `.md` file, named after the type.
- Module-level (free) functions are grouped together in a `functions.md` file per module.
- Each module directory contains an `index.md` listing its submodules, types, and functions.
- The root `index.md` is the entry point, linking to all top-level modules and items.

The specta_serde crate's top-level doc comment:
----
[Serde](https://serde.rs) support for Specta.

# Choosing a mode

- Use [`apply`] when serde behavior is symmetric and a single exported shape
  should work for both serialization and deserialization.
- Use [`apply_phases`] when serde behavior differs by direction (for example
  deserialize-widening enums, asymmetric conversion attributes, or explicit
  [`Phased`] overrides).

# `serde_with` and `#[serde(with = ...)]`

`serde_with` is supported through the same mechanism as raw serde codec
attributes because it expands to serde metadata (`with`, `serialize_with`,
`deserialize_with`).

When codecs change the wire type, add an explicit Specta override:

```rust,ignore
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Type, Serialize, Deserialize)]
struct Digest {
    #[serde(with = "hex_bytes")]
    #[specta(type = String)]
    value: Vec<u8>,
}
```

If serialize and deserialize shapes are different, use [`Phased`] and
[`apply_phases`].

This is required because a single unified type graph cannot represent two
different directional wire shapes at once.

```rust,ignore
use serde::{Deserialize, Serialize};
use serde_with::{OneOrMany, serde_as};
use specta::{Type, Types};

#[derive(Type, Serialize, Deserialize)]
#[serde(untagged)]
enum OneOrManyString {
    One(String),
    Many(Vec<String>),
}

#[serde_as]
#[derive(Type, Serialize, Deserialize)]
struct Filters {
    #[serde_as(as = "OneOrMany<_>")]
    #[specta(type = specta_serde::Phased<Vec<String>, OneOrManyString>)]
    tags: Vec<String>,
}

let types = Types::default().register::<Filters>();
let phased_types = specta_serde::apply_phases(types)?;
```

As an alternative to codec attributes, `#[serde(into = ...)]`,
`#[serde(from = ...)]`, and `#[serde(try_from = ...)]` often produce better
type inference because the wire type is modeled as an explicit Rust type:

```rust,ignore
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Type, Serialize, Deserialize)]
struct UserWire {
    id: String,
}

#[derive(Type, Clone, Serialize, Deserialize)]
#[serde(into = "UserWire")]
struct UserInto {
    id: String,
}

#[derive(Type, Clone, Serialize, Deserialize)]
#[serde(from = "UserWire")]
struct UserFrom {
    id: String,
}

#[derive(Type, Clone, Serialize, Deserialize)]
#[serde(try_from = "UserWire")]
struct UserTryFrom {
    id: String,
}
```

See `examples/basic-ts/src/main.rs` for a complete exporter example using
[`apply`] and [`apply_phases`].

