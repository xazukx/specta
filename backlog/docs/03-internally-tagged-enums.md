# Internally Tagged Enums with Variant Structs

## Status: Fully Supported

Specta fully supports serde's internally tagged enum representation (`#[serde(tag = "...")]`) including variants with struct fields. This is handled automatically through the serde transformation pipeline.

---

## How to Use

### Basic Example

```rust
use serde::Serialize;
use specta::Type;

#[derive(Type, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    /// A user was created
    UserCreated {
        user_id: String,
        email: String,
    },
    /// A user logged in
    UserLoggedIn {
        user_id: String,
        ip_address: String,
    },
    /// System started
    SystemStarted,
}
```

### TypeScript Output

```typescript
export type Event =
    | { type: "UserCreated"; user_id: string; email: string }
    | { type: "UserLoggedIn"; user_id: string; ip_address: string }
    | { type: "SystemStarted" };
```

The tag field (`"type"`) is merged directly into each variant's object. Unit variants become objects with only the tag field.

---

## Export Code

The key requirement is to run the types through `specta_serde` to apply serde transformations. Without this step, the tag field won't appear in the output.

```rust
use specta::Types;
use specta_typescript::Typescript;

fn main() {
    let types = Types::default()
        .register::<Event>();

    // IMPORTANT: Apply serde transformations to get correct tagged output
    let resolved = specta_serde::apply(types).unwrap();
    // Or for phase-specific (serialize vs deserialize) output:
    // let resolved = specta_serde::apply_phases(types).unwrap();

    Typescript::default()
        .export_to("./bindings.ts", &resolved)
        .unwrap();
}
```

### Without `specta_serde::apply()`

If you skip the serde transformation and export raw types:

```rust
let resolved = specta::ResolvedTypes::from_resolved_types(types);
```

The output will be a **raw union** without the tag field:

```typescript
// RAW — no serde transformation applied
export type Event = "UserCreated" | "UserLoggedIn" | "SystemStarted";
```

This is incorrect for internally tagged enums. Always use `specta_serde::apply()`.

---

## Variant Types

### Unit Variants

```rust
#[serde(tag = "kind")]
enum Status {
    Active,
    Inactive,
}
```

```typescript
export type Status = { kind: "Active" } | { kind: "Inactive" };
```

### Struct Variants (Named Fields)

```rust
#[serde(tag = "kind")]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}
```

```typescript
export type Shape =
    | { kind: "Circle"; radius: number }
    | { kind: "Rectangle"; width: number; height: number };
```

### Tuple Variants (Single Struct Payload)

Internally tagged enums allow tuple variants only with **exactly one field** that is a struct or map type. The tag field is merged with the payload:

```rust
#[derive(Type, Serialize)]
struct Point { x: f64, y: f64 }

#[derive(Type, Serialize)]
#[serde(tag = "type")]
enum Geometry {
    Point(Point),
}
```

```typescript
export type Geometry = { type: "Point" } & (Point);
```

**Restriction**: Tuple variants with primitive payloads (`String`, `i32`, etc.) are **not valid** for internally tagged enums in serde and specta will error during validation.

### Flattened Fields in Variants

```rust
#[derive(Type, Serialize)]
struct Metadata { created_at: String }

#[derive(Type, Serialize)]
#[serde(tag = "type")]
enum Item {
    Document {
        title: String,
        #[serde(flatten)]
        meta: Metadata,
    },
}
```

```typescript
export type Item = { type: "Document"; title: string } & (Metadata);
```

---

## Combining with Other Serde Attributes

### `rename_all`

```rust
#[derive(Type, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Action {
    CreateUser { user_name: String },
    DeleteAccount { account_id: i32 },
}
```

```typescript
export type Action =
    | { type: "createUser"; userName: string }
    | { type: "deleteAccount"; accountId: number };
```

### `rename` on Variants

```rust
#[derive(Type, Serialize)]
#[serde(tag = "type")]
enum Message {
    #[serde(rename = "text")]
    TextMessage { content: String },
    #[serde(rename = "image")]
    ImageMessage { url: String },
}
```

```typescript
export type Message =
    | { type: "text"; content: string }
    | { type: "image"; url: string };
```

### Mixed Tagged and Untagged Variants

Serde supports mixing tagged and `#[serde(untagged)]` variants:

```rust
#[derive(Type, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum MixedTaggedAndUntaggedStruct {
    Tagged { value: String },
    #[serde(untagged)]
    Raw { raw_value: String },
}
```

```typescript
export type MixedTaggedAndUntaggedStruct =
    | { kind: "Tagged"; value: string }
    | { raw_value: string };
```

### `#[serde(other)]` for Catch-All

```rust
#[derive(Type, Deserialize)]
#[serde(tag = "kind")]
enum InternalOther {
    #[serde(rename = "known")]
    Known,
    #[serde(other)]
    Other,
}
```

```typescript
export type InternalOther = { kind: "known" } | { kind: string };
```

The `#[serde(other)]` variant gets a `string` type tag instead of a literal, acting as a catch-all.

---

## Validation Rules

Specta validates internally tagged enums according to serde's rules:

| Variant Type | Allowed? | Notes |
|---|---|---|
| Unit (no fields) | Yes | Produces `{ tag: "Name" }` |
| Named fields (struct) | Yes | Tag merged with fields |
| Single-field tuple with struct payload | Yes | Tag merged via intersection `&` |
| Single-field tuple with map payload | Yes | Tag merged via intersection `&` |
| Multi-field tuple | No | Serde does not support this |
| Single-field tuple with primitive | No | Cannot merge tag into a primitive |
| Single-field tuple with list | No | Cannot merge tag into an array |

If you violate these rules, `specta_serde::apply()` will return an error describing the invalid variant.

---

## Comparison with Other Enum Representations

| Representation | Attribute | Example Output |
|---|---|---|
| External (default) | none | `{ "UserCreated": { ... } }` → `{ UserCreated: { user_id: string } }` |
| Internal | `#[serde(tag = "type")]` | `{ type: "UserCreated"; user_id: string }` |
| Adjacent | `#[serde(tag = "type", content = "data")]` | `{ type: "UserCreated"; data: { user_id: string } }` |
| Untagged | `#[serde(untagged)]` | `{ user_id: string }` (no discriminator) |

---

## Key Files

| File | Purpose |
|------|---------|
| `specta-serde/src/repr.rs` | `EnumRepr::Internal { tag }` definition |
| `specta-serde/src/lib.rs:1493-1557` | `transform_internal_variant()` — core serde transformation |
| `specta-serde/src/validate.rs:657-765` | Validation of internally tagged enum constraints |
| `specta-typescript/src/legacy.rs:372-521` | `enum_variant_datatype()` — TypeScript rendering |
| `specta-typescript/src/legacy.rs:588-624` | `variant_discriminator()` — tag field detection |
| `examples/basic-ts/src/main.rs:52-57` | Example with `SerdeInternalExample` |
