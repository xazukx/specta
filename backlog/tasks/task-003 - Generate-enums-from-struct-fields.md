---
id: TASK-003
title: Generate enums from struct fields
status: To Do
assignee: []
created_date: '2026-04-08 03:49'
labels: [macro, enum, companion]
dependencies: []
---

# Generate Enum Companions from Struct Fields

## Goal

Implement a `#[derive(TypeCompanion)]` macro that generates companion enums for
structs and variant-name arrays for enums, enabling dynamic field access and
updates at runtime. This is inspired by the
[`enum_companion`](https://github.com/solidev/enum_companion) crate but
integrated natively into the specta ecosystem so that the generated types can
also be exported via specta's language exporters (TypeScript, Zod, JSON Schema,
etc.).

---

## Design Decision: Where to Put the Code

**Chosen approach: extend `specta` + `specta-macros`** (not a new crate).

Rationale:
- The workspace already centralizes all derive macros in `specta-macros` and all
  core traits in `specta`. Adding a new derive macro here follows the established
  pattern (`Type`, `specta_const`, `specta`).
- The generated companion enums benefit from tight integration with `specta::Type`
  — they can optionally auto-derive `Type` so they appear in the type collection
  and are exportable without any extra work.
- A separate crate would add workspace complexity (another macro crate or a
  dependency cycle) for a feature that logically belongs alongside struct/enum
  metadata generation.

The feature will be gated behind a cargo feature flag `companion` on both crates,
so it adds zero overhead when unused.

---

## What Gets Generated

### For Structs

Given a struct:

```rust
#[derive(TypeCompanion)]
#[companion(derive_value(Debug, PartialEq))]
#[serde(rename_all = "camelCase")]
struct UserProfile {
    id: u64,
    #[companion(title = "Name")]
    #[serde(rename = "user_name")]
    username: String,
    #[companion(skip)]
    password_hash: String,
    display_name: Option<String>,
}
```

The macro generates:

#### 1. Static Field Names Array — `UserProfile::FIELD_NAMES`

A `&'static [&'static str]` array of the serialized field names, respecting
serde rename conventions (`rename`, `rename_all`) applied to the struct and its
fields.

```rust
impl UserProfile {
    pub const FIELD_NAMES: &'static [&'static str] = &[
        "id",
        "user_name",     // serde(rename = "user_name") applied
        "displayName",   // serde(rename_all = "camelCase") applied
    ];
    // password_hash is skipped
}
```

The names in this array reflect **how serde would serialize the field name**, not
the Rust identifier. This makes `FIELD_NAMES` directly usable for
serialization-aware lookups, form generation, and API field validation.

#### 2. Field Enum — `UserProfileField`

Variant names are derived from the serde-resolved field names, converted to
PascalCase. `#[serde(rename = "...")]` and `#[serde(rename_all = "...")]`
determine the variant names.

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UserProfileField {
    Id,
    UserName,      // from serde(rename = "user_name") → PascalCase
    DisplayName,   // from serde(rename_all = "camelCase") on "display_name" → PascalCase
    // password_hash is skipped
}
```

With automatic impls:
- `std::fmt::Display` (returns the field name string)
- `std::str::FromStr` (parses "id" | "Id" → `Id`, etc.)
- `specta::CompanionField` trait (see below)

#### 3. Value Enum — `UserProfileValue`

```rust
#[derive(Clone, Debug, PartialEq)]  // user-requested derives added
pub enum UserProfileValue {
    Id(u64),
    UserName(String),
    DisplayName(Option<String>),
}
```

With automatic impls:
- `specta::CompanionValue` trait (see below)
- `TryFrom<UserProfileValue> for T` for each unique field type
- `TryFrom<(UserProfileField, T)> for UserProfileValue` for each unique field type

#### 4. Methods on the Original Struct

```rust
impl UserProfile {
    pub fn fields() -> &'static [UserProfileField] { ... }
    pub fn field_names() -> &'static [&'static str] { Self::FIELD_NAMES }
    pub fn value(&self, field: UserProfileField) -> UserProfileValue { ... }
    pub fn update(&mut self, value: UserProfileValue) { ... }
    pub fn as_values(&self) -> Vec<UserProfileValue> { ... }
}
```

#### 5. Trait Implementations

```rust
impl specta::TypeCompanion<UserProfileField, UserProfileValue> for UserProfile { ... }
```

#### 6. Optional `specta::Type` Derivation

The generated Field and Value enums do **not** derive `specta::Type` by default.
To make them exportable, use `#[companion(derive_field(specta::Type))]` and/or
`#[companion(derive_value(specta::Type))]`.

### For Enums

Given an enum:

```rust
#[derive(TypeCompanion)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Status {
    Active,
    Inactive,
    #[companion(skip)]
    Internal,
    #[serde(rename = "on_hold")]
    OnHold,
}
```

The macro generates **only** a static variant names array — no `*Field` enum, no
`*Value` enum, and no `TypeCompanion` trait impl:

```rust
impl Status {
    /// Variant names as they would be serialized by serde.
    pub const VARIANT_NAMES: &'static [&'static str] = &[
        "ACTIVE",
        "INACTIVE",
        // Internal is skipped
        "on_hold",       // serde(rename) takes precedence
    ];

    pub fn variant_names() -> &'static [&'static str] { Self::VARIANT_NAMES }
}
```

The names follow serde rename conventions (`rename`, `rename_all`) just like
`FIELD_NAMES` does for structs.

---

## Traits to Add to `specta`

### `specta::CompanionField`

```rust
pub trait CompanionField: Copy + Clone + Eq + std::hash::Hash {
    /// The serde-resolved field name (e.g. "user_name").
    fn name(&self) -> &'static str;
    /// The Rust type as a string (e.g. "String").
    fn type_str(&self) -> &'static str;
    /// Human-readable title. Defaults to `name()`.
    fn title(&self) -> &'static str {
        self.name()
    }
    /// Ordering hint for UI rendering. Defaults to 0.
    fn order(&self) -> isize {
        0
    }
}
```

### `specta::CompanionValue`

```rust
pub trait CompanionValue {
    /// The field name this value corresponds to.
    fn field_name(&self) -> &'static str;
    /// The Rust type name of the inner value.
    fn type_name(&self) -> &'static str;
}
```

### `specta::TypeCompanion<F, V>`

```rust
pub trait TypeCompanion<F, V>
where
    F: Copy + 'static,
{
    /// Returns the value of a specific field.
    fn value(&self, field: F) -> V;
    /// Updates the value of a specific field.
    fn update(&mut self, value: V);
    /// Returns an array of all field enum variants.
    fn fields() -> &'static [F];
    /// Returns a vector of all field values.
    fn as_values(&self) -> Vec<V>;
}
```

---

## Attributes

### Container-Level (`#[companion(...)]`)

| Attribute | Type | Description |
|---|---|---|
| `derive_field(Trait, ...)` | list | Extra derives on the Field enum |
| `derive_value(Trait, ...)` | list | Extra derives on the Value enum |
| `serde_field(...)` | list | Serde attributes on the Field enum |
| `serde_value(...)` | list | Serde attributes on the Value enum |
| `value_fn = "name"` | string | Rename the `.value()` method |
| `update_fn = "name"` | string | Rename the `.update()` method |
| `fields_fn = "name"` | string | Rename the `.fields()` method |
| `collect = false` | bool | Skip auto-collection (when `collect` feature is on) |

### Field-Level (`#[companion(...)]`)

| Attribute | Type | Description |
|---|---|---|
| `skip` | flag | Exclude this field/variant entirely |
| `title = "Title"` | string | Human-readable title (for `CompanionField::title()`) |
| `order = N` | isize | Ordering hint (for `CompanionField::order()`) |

---

## Implementation Plan

### Phase 1: Core Traits in `specta`

**Files to create/modify:**

1. **`specta/Cargo.toml`** — Add `companion` feature flag (default off).
2. **`specta/src/companion.rs`** — Define `CompanionField`, `CompanionValue`, and
   `TypeCompanion` traits.
3. **`specta/src/lib.rs`** — Conditionally expose the `companion` module and
   re-export the traits when the `companion` feature is enabled.

### Phase 2: Derive Macro in `specta-macros`

**Files to create/modify:**

4. **`specta-macros/Cargo.toml`** — Add `companion` feature flag.
5. **`specta-macros/src/companion/mod.rs`** — Main derive entry point. Parse
   `DeriveInput`, dispatch to struct handler or enum handler (reject unions with
   a clear error).
6. **`specta-macros/src/companion/attr.rs`** — Attribute parsing for container-
   and field-level `#[companion(...)]` attributes. Follow the same pattern as
   `type/attr/container.rs` and `type/attr/field.rs`.
7. **`specta-macros/src/companion/field_enum.rs`** — Generate the `{Struct}Field`
   enum, its `FIELDS` const, `Display`, `FromStr`, and `CompanionField` impl.
8. **`specta-macros/src/companion/value_enum.rs`** — Generate the `{Struct}Value`
   enum, its `CompanionValue` impl, and `TryFrom` impls.
9. **`specta-macros/src/companion/struct_impl.rs`** — Generate the `impl Struct`
   block with `FIELD_NAMES`, `value()`, `update()`, `fields()`, `field_names()`,
   `as_values()`, and the `TypeCompanion` trait impl. Must read serde attributes
   (`rename`, `rename_all`) to compute the serialized field names for the
   `FIELD_NAMES` array.
10. **`specta-macros/src/companion/enum_impl.rs`** — Generate the `impl Enum`
    block with `VARIANT_NAMES` and `variant_names()`. Must read serde attributes
    (`rename`, `rename_all`) to compute the serialized variant names.
11. **`specta-macros/src/lib.rs`** — Register `#[proc_macro_derive(TypeCompanion,
    attributes(companion))]` gated behind `#[cfg(feature = "companion")]`.

### Phase 3: Collection Support

12. **`specta/src/collect.rs`** — (Optional) If `collect` + `companion` features
    are both on, auto-register the generated Field and Value enums into the type
    collection via `#[ctor]`, following the same pattern as the `Type` derive.

### Phase 4: Re-export and Feature Wiring

13. **`specta/src/lib.rs`** — Re-export `specta_macros::TypeCompanion` under the
    `derive` + `companion` feature gate.
14. **Workspace `Cargo.toml`** — No changes needed (workspace deps already cover
    all proc-macro utilities).

### Phase 5: Tests

15. **`tests/`** — Add integration tests:
    - Basic struct → generates Field + Value enums, methods work correctly.
    - `FIELD_NAMES` contains serde-serialized names.
    - `#[serde(rename_all = "camelCase")]` is reflected in `FIELD_NAMES`.
    - `#[serde(rename = "...")]` on individual fields is reflected in `FIELD_NAMES`.
    - Serde renames also determine Field/Value enum variant names (PascalCase conversion).
    - `#[companion(skip)]` excludes fields/variants from all generated output.
    - `title`, `order` metadata flows through `CompanionField`.
    - `TryFrom` conversions work and fail correctly.
    - `TypeCompanion` trait enables generic programming.
    - `derive_field(...)` / `derive_value(...)` applies extra derives.
    - `serde_field(...)` / `serde_value(...)` applies serde attributes.
    - Custom method names via `value_fn`, `update_fn`, `fields_fn`.
    - `derive_field(specta::Type)` / `derive_value(specta::Type)` makes generated enums exportable.
    - `collect = false` prevents auto-registration.
    - Enum input → generates `VARIANT_NAMES` with serde rename support.
    - Enum `#[companion(skip)]` excludes variants from `VARIANT_NAMES`.
    - Compile-fail tests: union input, tuple struct input.

### Phase 6: Example

16. **`examples/`** — Add or extend an example demonstrating:
    - Deriving `TypeCompanion` on a struct — using Field/Value enums and `FIELD_NAMES`.
    - Deriving `TypeCompanion` on an enum — using `VARIANT_NAMES`.
    - Using the generated enums for dynamic field access.
    - Exporting the companion enums to TypeScript/Zod/JSON Schema.

---

## Constraints and Edge Cases

- **All fields must implement `Clone`**: The `value()` method clones field
  values into the Value enum. This should be documented and enforced via
  where-bounds in the generated code.
- **Named fields only (for structs)**: Tuple structs and unit structs are not
  supported for the full companion generation. The macro should emit a clear
  error if applied to these.
- **Enums**: All enum variant kinds (unit, tuple, struct) are supported — only
  the variant names are extracted into `VARIANT_NAMES`. No `*Value` trait is
  generated for enums.
- **Generic structs**: Generic type parameters should be forwarded to the Value
  enum. For example, `struct Wrapper<T> { inner: T }` generates
  `enum WrapperValue<T> { Inner(T) }`. The `CompanionField` enum remains
  non-generic. `TryFrom` impls are skipped for generic/associated-type fields
  (Rust orphan rule limitation).
- **`Option<T>` fields**: Handled naturally — the Value enum variant wraps
  `Option<T>`.
- **Visibility**: The generated enums inherit the visibility of the original
  struct.
- **Naming collisions**: If the generated enum names (`{Struct}Field`,
  `{Struct}Value`) collide with existing types, the compiler will report the
  conflict naturally. No special handling needed.
- **Serde rename conventions**: `#[serde(rename_all = "...")]` and
  `#[serde(rename = "...")]` on the struct/enum and its fields/variants are read
  by the macro to compute `FIELD_NAMES` / `VARIANT_NAMES`. These represent the
  serialized names, not the Rust identifiers. The companion enum variant names
  (`{Struct}Field`, `{Struct}Value`) are also derived from the serde-resolved
  names, converted to PascalCase. There is no separate `#[companion(rename)]` —
  serde is the single source of truth for naming. Serde attributes on the
  *generated* companion enums themselves are controlled separately via
  `serde_field(...)` and `serde_value(...)`.

---

## Resolved Design Decisions

1. **No default `Type` derivation**: Generated enums do not derive `specta::Type`
   by default. Users opt in via `#[companion(derive_field(specta::Type))]` and
   `#[companion(derive_value(specta::Type))]`. There is no `#[companion(export)]`
   attribute.
2. **`TypeCompanion` trait bounds**: The trait uses `F: Copy + 'static` (no
   `CompanionField` bound), matching the `enum_companion` crate's
   `EnumCompanionTrait` pattern. `CompanionField` and `CompanionValue` are
   implemented on the generated enums independently, not required by the main
   trait.
3. **`#[specta(skip)]` vs `#[companion(skip)]`**: These are independent concerns.
   A field with `#[specta(skip)]` but no `#[companion(skip)]` still appears in
   the companion enums, and vice versa.
