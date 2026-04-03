# Documentation Summary

Crate: `specta_typescript`
Version: `0.0.11`

## How to use these docs

This directory contains Markdown documentation generated from rustdoc JSON.
The files are organized to mirror the crate's module structure:

- Each struct, enum, and trait has its own `.md` file, named after the type.
- Module-level (free) functions are grouped together in a `functions.md` file per module.
- Each module directory contains an `index.md` listing its submodules, types, and functions.
- The root `index.md` is the entry point, linking to all top-level modules and items.

The specta_typescript crate's top-level doc comment:
----
[TypeScript](https://www.typescriptlang.org) language exporter.

# Usage

Add `specta` and `specta-typescript` to your project:

```bash
cargo add specta@2.0.0-rc.24 --features derive,export
cargo add specta-typescript@0.0.11
cargo add specta-serde@0.0.11
```

Next copy the following into your `main.rs` file:

```rust
use specta::{ResolvedTypes, Type, Types};
use specta_typescript::Typescript;

#[derive(Type)]
pub struct MyType {
    pub field: MyOtherType,
}


#[derive(Type)]
pub struct MyOtherType {
    pub other_field: String,
}

let mut types = Types::default()
    // We don't need to specify `MyOtherType` because it's referenced by `MyType`
    .register::<MyType>();
let resolved_types = ResolvedTypes::from_resolved_types(types);

Typescript::default()
    .export_to("./bindings.ts", &resolved_types)
    .unwrap();
```

Now your setup with Specta!

If you get tired of listing all your types manually? Checkout `specta::collect`!


