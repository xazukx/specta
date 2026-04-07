# specta-serde — Serde Attribute Integration

## Status: Implemented

The `specta-serde` crate transforms Specta's type graph to reflect how serde actually serializes and deserializes Rust types. Without it, exported types represent raw Rust shapes — field renames, enum tagging, skipped fields, and other serde attributes are ignored.

---

## Why This Crate Exists

Serde attributes modify the wire representation of types. A Rust struct with `#[serde(rename_all = "camelCase")]` produces camelCase JSON keys, but Specta's raw `DataType` still uses the original Rust field names. Similarly, `#[serde(tag = "type")]` changes how enums are encoded, but Specta's raw enum representation has no tag field.

`specta-serde` bridges this gap. It takes the raw type graph (with embedded serde metadata from `specta-macros`) and rewrites it to match what serde would actually produce on the wire.

### Pipeline Position

```
#[derive(Type, Serialize, Deserialize)]  (user code)
        ↓
specta-macros                            (extracts serde attrs into Specta metadata)
        ↓
Types collection                         (raw type graph)
        ↓
specta-serde::apply / apply_phases       (rewrites types to match serde wire format)
        ↓
ResolvedTypes                            (final type graph)
        ↓
specta-typescript / specta-swift / etc.  (language export)
```

---

## Core API

### `apply(types: Types) -> Result<ResolvedTypes>`

Applies serde transformations in **unified mode** — produces a single type per Rust type that works for both serialization and deserialization. This is the default choice for most use cases.

```rust
use specta::Types;
use specta_serde::apply;

let types = Types::default()
    .register::<MyStruct>();

let resolved = apply(types).unwrap();
```

Returns an error if serde attributes create directional differences that can't be represented as a single shape (e.g., `#[serde(other)]`, asymmetric `from`/`into` conversions).

### `apply_phases(types: Types) -> Result<ResolvedTypes>`

Applies serde transformations in **split-phase mode**. When a type behaves differently during serialization vs. deserialization, this creates paired `TypeName_Serialize` and `TypeName_Deserialize` variants.

```rust
use specta_serde::apply_phases;

let resolved = apply_phases(types).unwrap();
```

Use this when your types include:
- `#[serde(other)]` on enum variants (widens the tag type only on deserialize)
- `#[serde(skip_serializing)]` / `#[serde(skip_deserializing)]` on different fields
- `#[serde(from = "A", into = "B")]` with different types
- `#[specta(type = specta_serde::Phased<SerializeType, DeserializeType>)]`

### `validate(dt: &DataType, types: &ResolvedTypes) -> Result<()>`

Validates whether a `DataType` is valid for serde export. Called automatically by `apply`/`apply_phases` for all named types. Useful when exporting individual types directly.

### `select_phase_datatype(dt: &DataType, types: &ResolvedTypes, phase: Phase) -> DataType`

After calling `apply_phases`, selects either the serialize or deserialize shape. Recursively rewrites references to use the appropriate phase variant.

```rust
use specta_serde::{select_phase_datatype, Phase};

let serialize_dt = select_phase_datatype(&dt, &resolved, Phase::Serialize);
let deserialize_dt = select_phase_datatype(&dt, &resolved, Phase::Deserialize);
```

---

## What It Transforms

### Field and Variant Renames

Applies serde's 8 rename rules (`camelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, etc.) and individual `#[serde(rename = "...")]` attributes.

```rust
#[derive(Type, Serialize)]
#[serde(rename_all = "camelCase")]
struct User {
    user_name: String,    // → "userName"
    email_address: String, // → "emailAddress"
}
```

### Enum Tagging

Supports all four serde enum representations:

| Representation | Attribute | Wire Format |
|---|---|---|
| External (default) | none | `{ "VariantName": payload }` |
| Internal | `#[serde(tag = "type")]` | `{ type: "VariantName", ...fields }` |
| Adjacent | `#[serde(tag = "t", content = "c")]` | `{ t: "VariantName", c: payload }` |
| Untagged | `#[serde(untagged)]` | `payload` (no discriminator) |

### Skipped Fields

- `#[serde(skip)]` — field removed entirely
- `#[serde(skip_serializing)]` — removed in serialize phase only
- `#[serde(skip_deserializing)]` — removed in deserialize phase only

### Flattening

`#[serde(flatten)]` merges a nested struct's fields into the parent, rendered as an intersection type in TypeScript.

### Conversion Attributes

- `#[serde(from = "WireType")]` — deserialize shape uses WireType
- `#[serde(into = "WireType")]` — serialize shape uses WireType
- `#[serde(try_from = "WireType")]` — same as `from` for type export purposes

### Transparent Structs

`#[serde(transparent)]` unwraps a single-field struct to its inner type.

---

## The `Phased<S, D>` Marker

For fields where the serialize and deserialize types differ intentionally, use `Phased`:

```rust
use specta_serde::Phased;

#[derive(Type, Serialize, Deserialize)]
struct Filters {
    #[specta(type = Phased<Vec<String>, OneOrManyString>)]
    tags: Vec<String>,
}
```

When exported with `apply_phases`, this produces:
- `Filters_Serialize` with `tags: Vec<String>`
- `Filters_Deserialize` with `tags: OneOrManyString`

If both type parameters resolve to the same type, it collapses to a single unified type.

---

## Error Handling

`apply()` and `apply_phases()` return detailed errors for invalid serde usage:

- Internally tagged enum with a tuple variant containing a primitive (serde doesn't support this)
- `#[serde(skip_serializing)]` and `#[serde(skip_deserializing)]` on the same field in unified mode
- Incompatible `from`/`into` conversions in unified mode
- Custom serde codecs (`serialize_with`, `deserialize_with`) without a `#[specta(type = ...)]` override
- Unresolved generic references in serde attributes

Errors include the type name, field path, and attribute that caused the issue.

---

## Key Files

| File | Purpose |
|------|---------|
| `specta-serde/src/lib.rs` | Main API (`apply`, `apply_phases`, `select_phase_datatype`), type rewriting logic |
| `specta-serde/src/phased.rs` | `Phased<S, D>` marker type for explicit phase splitting |
| `specta-serde/src/error.rs` | `Error` type with detailed error variants |
| `specta-serde/src/parser.rs` | Serde attribute parsing (`SerdeContainerAttrs`, `SerdeFieldAttrs`, `SerdeVariantAttrs`) |
| `specta-serde/src/validate.rs` | Validation logic for unified and phased modes |
| `specta-serde/src/repr.rs` | `EnumRepr` — the four serde enum tagging modes |
| `specta-serde/src/inflection.rs` | `RenameRule` — serde's 8 case-conversion rules |
