# specta-tags — Client-Side Runtime Type Transformations

## Status: Implemented

The `specta-tags` crate analyzes Specta types and generates inline JavaScript code that converts JSON values into their proper rich JavaScript types at runtime. It solves the problem of Rust types that serialize to JSON-compatible representations but need to be restored to richer types on the client (e.g., `BigInt`, `Date`, `Uint8Array`).

---

## Why This Crate Exists

JSON has a limited type system. When Rust types are serialized to JSON and sent over the wire, certain values lose their type fidelity:

| Rust Type | JSON Representation | Desired JS Type |
|---|---|---|
| `u64`, `i64`, `u128`, `i128` | string or number | `BigInt` |
| `chrono::DateTime<Utc>` | string | `Date` |
| `bytes::Bytes` | array of numbers | `Uint8Array` |

The TypeScript type system can declare these correctly (via branded types or type aliases), but the **runtime values** arriving from an API are still plain strings/numbers/arrays. Client code must manually convert them — or forget to, introducing subtle bugs.

`specta-tags` automates this by analyzing the type graph and generating a JavaScript transformation function that converts all tagged fields in a value to their correct types.

---

## Core API

### `TransformPlan`

The main entry point. Analyzes a `DataType` and builds a plan for transforming values of that type.

```rust
use specta::{Type, Types};
use specta_tags::TransformPlan;

#[derive(Type)]
struct Event {
    id: u128,
    timestamp: chrono::DateTime<chrono::Utc>,
    data: bytes::Bytes,
}

let mut types = Types::default();
let dt = Event::definition(&mut types);
let resolved = specta::ResolvedTypes::from_resolved_types(types);

let plan = TransformPlan::analyze(&dt, &resolved);
let js = plan.map("value");
```

`plan.map("value")` returns a JavaScript expression that transforms the input variable. If no transformations are needed, it returns the variable name unchanged.

### `Tag`

Identifies which transformation to apply to a value:

```rust
pub enum Tag {
    BigInt,       // BigInt(value)
    Uint8Array,   // new Uint8Array(value)
    Date,         // new Date(value)
    Custom(Arc<dyn Fn(&str) -> Cow<'static, str> + Send + Sync>),
}
```

`Custom` allows defining your own transformation functions for types not covered by the builtins.

---

## Supported Types

### BigInt (64+ bit integers)

- `u64`, `i64`, `u128`, `i128`

### Date (datetime types)

- `std::time::SystemTime`
- `chrono::DateTime`, `chrono::NaiveDateTime`, `chrono::NaiveDate`, `chrono::Date`
- `time::PrimitiveDateTime`, `time::OffsetDateTime`, `time::Date`
- `jiff::Timestamp`, `jiff::Zoned`, `jiff::civil::Date`, `jiff::civil::DateTime`
- `toml::value::Datetime`
- `bson::DateTime`

### Uint8Array (byte buffers)

- `bytes::Bytes`, `bytes::BytesMut`

---

## Generated JavaScript

The crate generates self-contained JavaScript expressions (IIFEs) that apply conversions only to fields that need them. The output uses immutable-style updates — spreading unchanged fields and replacing only the converted ones.

### Simple Example

Given:
```rust
#[derive(Type)]
struct Data {
    count: u128,
    name: String,
}
```

`plan.map("value")` produces something like:
```javascript
(() => {
    let __value1 = value;
    {
        const next = BigInt(__value1["count"]);
        if (next !== __value1["count"]) __value1 = { ...__value1, "count": next };
    }
    return __value1;
})()
```

The `name` field is untouched because `String` needs no conversion.

### Nested and Container Types

The crate handles:
- **Nullable types** — conditional conversion with null checks
- **Lists** — `.map()` over each element
- **Maps** — `Object.entries()` + rebuild
- **Tuples** — positional conversion
- **Nested structs** — recursive descent
- **Enums** — variant matching using tag fields, field presence, or direct shape

### No-Op Optimization

If a type has no fields requiring conversion, `plan.map("value")` returns `Cow::Borrowed("value")` — no wrapper code generated.

---

## Usage with Exporters

`specta-tags` is designed to complement type exporters. A typical workflow:

1. Export types with `specta-typescript` (produces type declarations)
2. Analyze types with `specta-tags` (produces runtime converters)
3. Wire the converters into your API client layer

```rust
use specta::{Type, Types};
use specta_tags::TransformPlan;

fn generate_converter<T: Type>() -> String {
    let mut types = Types::default();
    let dt = T::definition(&mut types);
    let resolved = specta::ResolvedTypes::from_resolved_types(types);

    let plan = TransformPlan::analyze(&dt, &resolved);
    let body = plan.map("data");
    format!("function transform(data: unknown) {{ return {}; }}", body)
}
```

---

## How It Works Internally

The crate operates in three phases:

1. **Analysis** — walks the `DataType` tree and builds a `PlanNode` tree. Each node describes what (if any) conversion is needed at that position. Uses a builtin lookup table that matches module paths and type names to `Tag` values. Tracks visited types to prevent infinite recursion with recursive types.

2. **Planning** — the `PlanNode` enum models the structure:
   - `Identity` — no conversion needed
   - `Leaf(Tag)` — direct conversion (e.g., `BigInt(x)`)
   - `Nullable`, `List`, `Map`, `Tuple` — container wrappers
   - `Object` — struct with named or flattened fields
   - `Enum` — tagged union with variant matchers

3. **Rendering** — converts the plan into a JavaScript expression string. Uses unique variable names (`__value1`, `__item2`) and IIFEs for safe scoping. Only emits code for fields that actually need conversion.

---

## Key Files

| File | Purpose |
|------|---------|
| `specta-tags/src/lib.rs` | Full implementation: `TransformPlan`, `Tag`, analyzer, renderer, plan nodes |
| `specta-tags/Cargo.toml` | Dependencies: `specta`, `serde`, `serde_json` |
| `specta-tags/examples/tags.rs` | Basic usage example |
