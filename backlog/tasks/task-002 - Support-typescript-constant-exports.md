---
id: TASK-002
title: Support typescript constant exports
status: To Do
assignee: []
created_date: '2026-04-05 19:01'
labels: [feature, typescript, macros]
dependencies: []
---

# Support TypeScript Constant Exports

## Goal

Enable exporting Rust `const` and `static` values as TypeScript `export const` declarations, driven by a proc-macro attribute. The user should be able to control the output format (string, byte array, numeric, etc.), set a custom TypeScript variable name, and have specta automatically traverse nested unit/newtype structs to extract the underlying constant value.

## Desired User-Facing API

### Basic usage

```rust
use specta::Constant;

// Or, more directly on a const item (if proc-macro attribute on const is feasible):
#[specta_const]
pub const MAX_RETRIES: u32 = 5;
// Outputs: export const MAX_RETRIES = 5 as const;

#[specta_const(name = "MAX_RETRY_COUNT")]
pub const MAX_RETRIES: u32 = 5;
// Outputs: export const MAX_RETRY_COUNT = 5 as const;
```

### Output format control

```rust
// Default: infer from type (strings stay strings, numbers stay numbers)
#[specta_const]
pub const API_KEY_PREFIX: &str = "sk_live_";
// Outputs: export const API_KEY_PREFIX = "sk_live_" as const;

// Explicit string output (useful for byte arrays displayed as strings)
#[specta_const(output = "string")]
pub const MAGIC_HEADER: &[u8; 4] = b"RIFF";
// Outputs: export const MAGIC_HEADER = "RIFF" as const;

// Byte array output (Uint8Array literal)
#[specta_const(output = "bytes")]
pub const MAGIC_BYTES: &[u8; 4] = b"\x89PNG";
// Outputs: export const MAGIC_BYTES = new Uint8Array([137, 80, 78, 71]) as const;

// Byte array for regular arrays too
#[specta_const(output = "bytes")]
pub const HEADER: [u8; 3] = [0xFF, 0xD8, 0xFF];
// Outputs: export const HEADER = new Uint8Array([255, 216, 255]) as const;
```

### Nested unit struct traversal

When a constant references a newtype/unit struct chain, specta should unwrap it to find the underlying primitive value.

```rust
#[derive(Constant)]
pub struct Timeout(u64);
impl Timeout {
    pub const VALUE: u64 = 30_000;
}

#[derive(Constant)]
pub struct DefaultTimeout(Timeout);
// Specta should follow: DefaultTimeout -> Timeout -> u64
// and resolve the value from Timeout::VALUE
// Outputs: export const DefaultTimeout = 30000 as const;

// With explicit value and name override
#[derive(Constant)]
#[specta(name = "DEFAULT_TIMEOUT_MS", value = "Timeout::VALUE")]
pub struct DefaultTimeout;
// Outputs: export const DEFAULT_TIMEOUT_MS = 30000 as const;
```

## Architecture

### 1. New Data Representation: `ConstantValue`

Add a new type to `specta/src/datatype/` to represent constant values at runtime:

```rust
// specta/src/datatype/constant.rs

/// A compile-time constant value that can be exported by language exporters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstantValue {
    /// A string constant: `export const X = "hello" as const;`
    String(Cow<'static, str>),
    /// A numeric integer constant: `export const X = 42 as const;`
    Integer(i128),
    /// An unsigned integer constant: `export const X = 42 as const;`
    UnsignedInteger(u128),
    /// A floating point constant: `export const X = 3.14 as const;`
    Float(FloatBits),  // f64 bits stored as u64 for Eq/Hash
    /// A boolean constant: `export const X = true as const;`
    Bool(bool),
    /// A byte array constant: `export const X = new Uint8Array([...]) as const;`
    Bytes(Cow<'static, [u8]>),
    /// A null constant: `export const X = null as const;`
    Null,
}
```

### 2. Create a Parallel Collection

**Parallel `Constants` collection**

Keep constants separate from types. Add a new collection alongside `Types`:

```rust
// specta/src/constants.rs

/// A named constant that can be exported.
pub struct NamedConstant {
    /// Export name (the TypeScript variable name).
    pub name: Cow<'static, str>,
    /// The resolved constant value.
    pub value: ConstantValue,
    /// Documentation comments.
    pub docs: Cow<'static, str>,
    /// Deprecation metadata.
    pub deprecated: Option<Deprecated>,
    /// Source module path.
    pub module_path: Cow<'static, str>,
}

/// Collection of constants to export, analogous to `Types`.
#[derive(Default)]
pub struct Constants(Vec<NamedConstant>);

impl Constants {
    pub fn register<C: Constant>(&mut self) -> &mut Self { ... }
    pub fn into_sorted_iter(&self) -> impl Iterator<Item = &NamedConstant> { ... }
}
```

### 3. The `Constant` Trait

```rust
// specta/src/constant.rs

/// Trait for types that represent a compile-time constant exportable to TypeScript.
pub trait Constant {
    /// The name to use in the TypeScript export.
    fn name() -> Cow<'static, str>;

    /// The resolved constant value.
    fn value() -> ConstantValue;

    /// Register this constant into a Constants collection.
    fn register(constants: &mut Constants) {
        constants.push(NamedConstant {
            name: Self::name(),
            value: Self::value(),
            docs: Cow::Borrowed(""),
            deprecated: None,
            module_path: Cow::Borrowed(""),
        });
    }
}
```

### 4. Proc-Macro: `#[derive(Constant)]` and `#[specta_const]`

#### `#[derive(Constant)]` for structs

Located in `specta-macros/src/constant/`. Supports:

- **Unit structs** with a `const VALUE` associated constant.
- **Newtype structs** where the inner type also implements `Constant` (enabling traversal).
- Attributes:
  - `#[specta(name = "TS_NAME")]` - Override the TypeScript export name. Default: the Rust struct name converted to SCREAMING_SNAKE_CASE.
  - `#[specta(output = "string")]` or `#[specta(output = "bytes")]` - Control how the value is serialized. Default: inferred from the Rust type.
  - `#[specta(value = "SomeType::CONST")]` - Explicitly specify which constant expression to use. Default: `Self::VALUE` for unit structs, or traverse inner type for newtypes.

**Traversal logic for newtypes:**

```
1. If struct has #[specta(value = "expr")], use that expression directly.
2. If struct is a unit struct (no fields):
   a. Look for Self::VALUE associated constant.
   b. Compile error if not found.
3. If struct is a newtype (single field):
   a. Check if the inner type implements Constant.
   b. If yes, delegate: <InnerType as Constant>::value()
   c. If no, look for Self::VALUE.
   d. Compile error if neither works.
4. For deeper nesting, step 3 recurses naturally through the Constant trait.
```

#### `#[specta_const]` for bare `const` items

An attribute macro (not derive) that wraps a `const` item:

```rust
// Input:
#[specta_const(name = "MAX_RETRIES")]
pub const MAX_RETRIES: u32 = 5;

// Expands roughly to:
pub const MAX_RETRIES: u32 = 5;

// Plus a hidden type + Constant impl:
#[doc(hidden)]
pub struct __specta_const_MAX_RETRIES;
impl specta::Constant for __specta_const_MAX_RETRIES {
    fn name() -> Cow<'static, str> { Cow::Borrowed("MAX_RETRIES") }
    fn value() -> ConstantValue { ConstantValue::Integer(MAX_RETRIES as i128) }
}
// Plus inventory/linkme registration if using a global collection pattern.
```

### 5. TypeScript Exporter Changes

In `specta-typescript/src/`:

#### New function: `export_constant`

```rust
// specta-typescript/src/primitives.rs (or a new constants.rs)

pub fn export_constant(
    exporter: &Exporter,
    constant: &NamedConstant,
) -> Result<String, Error> {
    let value_str = match &constant.value {
        ConstantValue::String(s) => format!("\"{}\"", escape_typescript_string_literal(s)),
        ConstantValue::Integer(n) => n.to_string(),
        ConstantValue::UnsignedInteger(n) => n.to_string(),
        ConstantValue::Float(bits) => format_float(bits),
        ConstantValue::Bool(b) => b.to_string(),
        ConstantValue::Bytes(bytes) => {
            let inner = bytes.iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("new Uint8Array([{}])", inner)
        }
        ConstantValue::Null => "null".to_string(),
    };

    Ok(format!("export const {} = {} as const;", constant.name, value_str))
}
```

#### Integration with the exporter pipeline

The `Typescript` and `JSDoc` exporters should accept both `Types` and `Constants`, emitting type declarations first, then constant declarations:

```rust
// In the exporter's main output function:
for ndt in types.into_sorted_iter() {
    // ... existing type export logic ...
}

for constant in constants.into_sorted_iter() {
    writeln!(output, "{}", export_constant(exporter, constant)?)?;
}
```

### 6. Supported Rust Types and Their Mappings

| Rust Type | Default Output | `output = "string"` | `output = "bytes"` |
|---|---|---|---|
| `&str`, `String` | `"value"` | `"value"` | N/A (error) |
| `u8..u128`, `i8..i128`, `usize`, `isize` | `42` | `"42"` | N/A (error) |
| `f32`, `f64` | `3.14` | `"3.14"` | N/A (error) |
| `bool` | `true`/`false` | `"true"`/`"false"` | N/A (error) |
| `&[u8; N]`, `[u8; N]`, `Vec<u8>` | `new Uint8Array([...])` | `"decoded_string"` | `new Uint8Array([...])` |
| `()` | `null` | `"null"` | N/A (error) |

### 7. Nested Unit Struct Traversal - Detailed Design

The traversal is resolved **at compile time** through the `Constant` trait. No runtime reflection needed.

```rust
// Example chain:
struct Milliseconds(u64);
impl Milliseconds { const VALUE: u64 = 0; } // unused default

struct Timeout(Milliseconds);

struct Config {
    // not relevant - only newtypes/unit structs participate in const traversal
}

// derive(Constant) on Timeout generates:
impl Constant for Timeout {
    fn name() -> Cow<'static, str> { Cow::Borrowed("TIMEOUT") }
    fn value() -> ConstantValue {
        // Inner type is Milliseconds which impl Constant
        <Milliseconds as Constant>::value()
    }
}
```

**Important constraint:** The traversal chain must terminate at a type with either:
- A `const VALUE` that maps to a primitive (`str`, numeric, bool, byte slice).
- An explicit `#[specta(value = "...")]` expression.

If the chain reaches a type with no `Constant` impl and no `VALUE` constant, the derive macro should emit a clear compile error:

```
error: Cannot derive Constant for `Timeout`: inner type `Milliseconds` does not implement
       `specta::Constant` and no `#[specta(value = "...")]` was specified.
```

## File Changes Summary

| File | Change |
|---|---|
| `specta/src/datatype/constant.rs` | **New.** `ConstantValue` enum. |
| `specta/src/datatype.rs` | Re-export `ConstantValue`. |
| `specta/src/constant.rs` | **New.** `Constant` trait. |
| `specta/src/constants.rs` | **New.** `Constants` collection (parallel to `Types`). |
| `specta/src/lib.rs` | Export `Constant`, `Constants`, `ConstantValue`. |
| `specta-macros/src/constant/mod.rs` | **New.** `#[derive(Constant)]` implementation. |
| `specta-macros/src/constant/attr.rs` | **New.** Attribute parsing (`name`, `output`, `value`). |
| `specta-macros/src/lib.rs` | Register the new derive macro and `#[specta_const]` attribute macro. |
| `specta-typescript/src/constants.rs` | **New.** `export_constant()` rendering logic. |
| `specta-typescript/src/primitives.rs` | Import and integrate constant export in the pipeline. |
| `specta-typescript/src/exporter.rs` | Accept `Constants` alongside `Types` in export methods. |
| `tests/tests/constants.rs` | **New.** Tests for constant export. |

## Test Cases

1. **Primitive constants:** `const X: u32 = 42` -> `export const X = 42 as const;`
2. **String constants:** `const S: &str = "hello"` -> `export const S = "hello" as const;`
3. **Bool constants:** `const B: bool = true` -> `export const B = true as const;`
4. **Byte array as bytes:** `[0xFF, 0x00]` with `output = "bytes"` -> `export const X = new Uint8Array([255, 0]) as const;`
5. **Byte array as string:** `b"PNG"` with `output = "string"` -> `export const X = "PNG" as const;`
6. **Name override:** `#[specta(name = "API_URL")]` -> `export const API_URL = ...`
7. **Unit struct with VALUE:** struct + associated const -> correctly exported.
8. **Newtype traversal (1 level):** `struct Wrapper(Inner)` where `Inner` has VALUE -> resolves inner value.
9. **Newtype traversal (2+ levels):** `struct A(B)` where `B(C)` where `C` has VALUE -> resolves through chain.
10. **Compile error on missing VALUE:** unit struct without VALUE or Constant impl -> clear error message.
11. **String escaping:** values with quotes, backslashes, newlines -> properly escaped in TS output.
12. **Integration with Typescript exporter:** constants appear after type declarations in output.
13. **JSDoc mode:** constants get JSDoc comments when docs are present.
14. **Deprecated constants:** `#[deprecated]` -> JSDoc `@deprecated` annotation in output.
