# Zod Exporter: Current State and v4 Implementation Plan

## Status: Research / Backlog

---

## Current State

The specta-zod exporter is a **mature, well-tested implementation** targeting **Zod v3** API. It generates valid Zod v3 schemas from Rust types and covers all core Zod constructs. No version-specific configuration exists -- the output is version-agnostic using stable Zod v3 APIs.

### Currently Supported Zod Constructs

| Category | Constructs |
|---|---|
| **Primitives** | `z.number()`, `z.string()`, `z.boolean()`, `z.bigint()` |
| **Collections** | `z.array()`, `z.tuple()`, `z.object()`, `z.record()` |
| **Modifiers** | `.nullable()`, `.optional()`, `.and()`, `.strict()` |
| **Advanced** | `z.union()`, `z.literal()`, `z.lazy()`, `z.never()`, `z.unknown()`, `z.any()` |
| **Type inference** | `z.infer<typeof Schema>` |

### Rust to Zod Mapping

| Rust Type | Zod Output |
|---|---|
| `bool` | `z.boolean()` |
| `String`, `&str`, `char` | `z.string()` |
| `i8..i32`, `u8..u32`, `f16..f128` | `z.number()` |
| `i64`, `u64`, `i128`, `u128`, `isize`, `usize` | Configurable via `BigIntExportBehavior` |
| `Vec<T>` | `z.array(T)` |
| `[T; N]` | `z.tuple([T, T, ...])` |
| `(T1, T2)` | `z.tuple([T1, T2])` |
| `HashMap<K, V>` | `z.record(K, V)` |
| `Option<T>` | `T.nullable()` |
| Named struct | `z.object({ ... })` |
| Unit struct | `z.null()` |
| Enum (multiple variants) | `z.union([...])` |
| Enum (unit variant) | `z.literal("Variant")` |
| Recursive types | `z.lazy(() => Schema)` |
| Generic types | Factory functions with generic parameters |

### Configuration Options

```rust
Zod::new()
    .framework_prelude("import { z } from \"zod\";")  // Custom import
    .bigint(BigIntExportBehavior::BigInt)               // BigInt handling
    .layout(Layout::Files)                              // Output layout
    .header("// Custom header")                         // File header
    .framework_runtime(|..| { ... })                    // Custom runtime builder
```

**BigIntExportBehavior:** `Fail` (default) | `String` | `Number` | `BigInt`

**Layout:** `FlatFile` (default) | `ModulePrefixedName` | `Files`

### Opaque/Custom Types

- `specta_zod::Any<T>` --> `z.any()`
- `specta_zod::Unknown<T>` --> `z.unknown()`
- `specta_zod::Never<T>` --> `z.never()`
- `specta_zod::define(raw_expr)` --> Custom Zod expressions (arbitrary strings)

### Known TODOs in Code

- `Cargo.toml`: `// TODO: Don't depend on serde` -- specta-serde is an unnecessary dependency
- `Cargo.toml`: `// TODO: Remove this` -- thiserror dependency should be removed

---

## Zod v3 vs v4 Compatibility Analysis

The current output is **mostly compatible with Zod v4** since v4 is largely backward-compatible. However, several Zod v3 APIs used by specta-zod are **deprecated or changed in v4**:

### Breaking / Deprecated API Usage

| specta-zod uses | Zod v4 status | Required change |
|---|---|---|
| `.strict()` on objects | **Deprecated** -- use `z.strictObject()` | Replace `.strict()` with `z.strictObject()` wrapper |
| `.and()` for flattened fields | **Still works** but discouraged | Consider alternative merging strategies |
| `z.record(K, V)` (two args) | **Works** -- single-arg form dropped, two-arg required | Already correct |
| `z.literal()` | **Works** -- drops `symbol` support | No impact (Rust has no symbol type) |
| `z.union()` | **Works** unchanged | No change needed |
| `z.lazy()` | **Works** unchanged | No change needed |
| `.nullable()` | **Works** unchanged | No change needed |
| `.optional()` | **Works** unchanged | No change needed |

### New Zod v4 Features Not Yet Leveraged

| Zod v4 Feature | Potential specta-zod use |
|---|---|
| `z.strictObject()` | Replace `.strict()` for empty named structs |
| `z.looseObject()` | New option for passthrough-style objects |
| `z.enum(NativeEnum)` | `z.nativeEnum()` is deprecated; `z.enum()` now accepts native enums directly |
| `z.email()`, `z.uuid()`, etc. | Top-level string format validators (if specta adds format annotations) |
| `z.int()` | More specific integer validation (safe integers only) |
| `z.partialRecord()` | Better representation for partial record types |
| `error` parameter | Replaces `message` for error customization |
| `z.transform()` standalone | Could simplify certain transform patterns |

### Key Zod v4 Changes (from changelog)

- **Error customization**: Unified `error` param replaces `message`, `invalid_type_error`, `required_error`, and `errorMap`
- **`z.object()` behavior**: Defaults applied within optional fields; `.strict()`/`.passthrough()` deprecated in favor of `z.strictObject()`/`z.looseObject()`
- **`z.nativeEnum()` deprecated**: Merged into `z.enum()` which now accepts native enum inputs
- **`z.number().int()`**: Now only accepts safe integers; new `z.int()` top-level helper
- **`z.record()`**: Single-arg form dropped (already correct in specta-zod), enum keys now required (non-partial by default)
- **`z.string()` formats**: `.email()`, `.uuid()`, etc. deprecated as methods; moved to top-level `z.email()`, `z.uuid()`
- **Internal restructure**: `._def` moved to `._zod.def`; `ZodEffects` dropped; refinements now live as "checks" inside schemas; `ZodTransform` is new dedicated class
- **`z.unknown()`/`z.any()`**: No longer marked as "key optional" in inferred object types

---

## Implementation Plan: Zod v4 Support

### Phase 1: Version Configuration (Priority: High)

1. **Add `ZodVersion` enum** to specta-zod configuration:
   ```rust
   pub enum ZodVersion { V3, V4 }
   ```
2. **Add `.zod_version()` builder method** to the `Zod` exporter struct
3. **Default to v3** for backward compatibility, allow opting into v4

### Phase 2: Replace Deprecated APIs (Priority: High)

4. **Replace `.strict()` with `z.strictObject()`** when targeting v4
   - Currently used for empty named structs/variants
   - In v4 mode: wrap the shape in `z.strictObject({ ... })` instead of `z.object({ ... }).strict()`
5. **Update enum handling**: When targeting v4 and generating native enum schemas, use `z.enum(EnumName)` instead of `z.nativeEnum(EnumName)` (if this pattern is used)
6. **Update import statement** in framework prelude if v4 requires a different import path

### Phase 3: Leverage v4 Features (Priority: Medium)

7. **Add `z.int()` support**: For Rust integer types, optionally emit `z.int()` instead of `z.number()` in v4 mode -- provides built-in safe integer validation
8. **Add `z.partialRecord()` support**: For optional record types where keys are from an enum, use `z.partialRecord()` instead of `z.record()` with optional values
9. **Add top-level string format validators**: If specta types carry format metadata (email, uuid, url), emit `z.email()`, `z.uuid()`, `z.url()` instead of `z.string()` in v4 mode
10. **Consider `z.looseObject()`**: Add a passthrough object option for structs that should accept additional properties

### Phase 4: Testing and Validation (Priority: High)

11. **Add v4-specific snapshot tests** alongside existing v3 tests
12. **Validate generated schemas** actually parse correctly with both zod@3 and zod@4 npm packages
13. **Update documentation** with v4 configuration examples

### Phase 5: Cleanup (Priority: Low)

14. **Remove `thiserror` dependency** (noted TODO in Cargo.toml)
15. **Remove `specta-serde` dependency** if possible (noted TODO in Cargo.toml)
