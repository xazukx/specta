# JSON Schema Exporter: Current State

## Status: Active Development

---

## Current State

The specta-jsonschema exporter is **functional and covers most common use cases**. It supports core JSON Schema constructs, three draft versions, all four serde enum tagging modes, flattened fields, `$id`, `oneOf`, `additionalProperties`, field descriptions, external references, and proper inline/generic type resolution.

### Schema Versions Supported

| Version | `$schema` URI | Definitions Key |
|---|---|---|
| **Draft 7** (default) | `http://json-schema.org/draft-07/schema#` | `definitions` |
| **Draft 2019-09** | `https://json-schema.org/draft/2019-09/schema` | `$defs` |
| **Draft 2020-12** | `https://json-schema.org/draft/2020-12/schema` | `$defs` |

### Supported Constructs

#### Primitives (fully supported)

All Rust integer (i8–i128, u8–u128, isize, usize), float (f16, f32, f64, f128), bool, char, and str types are mapped to appropriate JSON Schema types with format and min/max annotations.

#### Collections

| Rust Type | JSON Schema Output |
|---|---|
| `Vec<T>` | `{"type": "array", "items": <T>}` |
| `[T; N]` | `{"type": "array", "items": <T>, "minItems": N, "maxItems": N}` |
| `(T1, T2)` | `{"type": "array", "prefixItems": [...], "minItems": N, "maxItems": N, "items": false}` |
| `HashMap<K, V>` | `{"type": "object", "additionalProperties": <V>}` |
| `()` | `{"type": "null"}` |

#### Structs

| Rust Type | JSON Schema Output |
|---|---|
| Unit struct | `{"type": "null"}` |
| Tuple struct | `{"type": "array", "prefixItems": [...]}` |
| Named struct | `{"type": "object", "properties": {...}, "required": [...], "additionalProperties": false}` |
| Struct with `#[serde(flatten)]` | `{"allOf": [{own fields}, {$ref to flattened type}]}` (no additionalProperties: false) |

#### Enums (all serde tagging modes via specta-serde)

| Pattern | JSON Schema Output |
|---|---|
| String-only (multi) | `{"enum": ["A", "B", "C"]}` |
| Single literal (tag) | `{"const": "VariantName"}` |
| External tagging | `{"anyOf": [...variants...]}` |
| Internal tagging | `{"anyOf": [{object with merged tag field}, ...]}` |
| Adjacent tagging | `{"anyOf": [{object with tag + content fields}, ...]}` |
| Untagged | `{"anyOf": [<raw type 1>, <raw type 2>]}` |

#### Nullable Types

| Rust Type | JSON Schema Output |
|---|---|
| `Option<T>` | `{"anyOf": [<T>, {"type": "null"}]}` |

### Configuration Options

```rust
JsonSchema::new()
    .schema_version(SchemaVersion::Draft202012)  // Target draft version
    .layout(Layout::Files)                        // SingleFile or Files
    .title("My Schema")                           // Root title
    .description("Description")                   // Root description
    .base_uri("https://example.com/schemas")      // $id on root; absolute URIs in Files
    .one_of(true)                                 // Use oneOf instead of anyOf
    .external_ref("TypeName", "https://...")       // External $ref override
```

### Reference Handling

| JSON Schema Keyword | Supported? | Notes |
|---|---|---|
| `$ref` (fragment) | **Yes** | `#/definitions/Name` or `#/$defs/Name` |
| `$ref` (file-relative) | **Yes** | `./Name.schema.json` in Files layout |
| `$ref` (absolute URI) | **Yes** | When `base_uri` is set in Files layout |
| `$ref` (external) | **Yes** | Via `.external_ref()` configuration |
| `$id` (root) | **Yes** | Via `.base_uri()` configuration |
| `$id` (per file) | **Yes** | In Files layout with `base_uri` |
| `definitions` (Draft 7) | **Yes** | Automatic based on schema version |
| `$defs` (2019-09+) | **Yes** | Automatic based on schema version |
| Inline types | **Yes** | `String`, `Vec<T>`, etc. resolve directly, never as `$ref` |
| Generic references | **Yes** | Resolved via `resolve_generics` when inlining |
| Circular references | **Yes** | Self-referential types use `$ref` naturally |

### Struct Field Features

- **`additionalProperties: false`** — emitted on all object schemas that don't use `#[serde(flatten)]`
- **`title`** — on every definition, from the Rust type name
- **`description`** — on definitions from type-level doc comments; on properties from field-level doc comments
- **`allOf`** — for structs with `#[serde(flatten)]` fields

### Test Coverage (45 tests)

- Basic export, schema versions, primitives, nullable, enums
- String enum optimization (`enum` array form, `const` for single literals)
- Title/description on definitions
- All four enum tagging modes (internal, adjacent, untagged, external) with serde
- Flattened struct fields with `allOf`
- Inline types not producing `$ref` or appearing in definitions
- References between structs and in enum variants
- `serde(rename_all)` (camelCase, SCREAMING_SNAKE_CASE)
- `$id` / `base_uri` configuration
- Files layout reference paths
- External `$ref` overrides
- `additionalProperties: false` with and without flatten
- `oneOf` option
- Circular / self-referential types
- Import `const` handling
- Field-level descriptions
- 10+ snapshot tests for stable output verification

---

## Implementation Status Summary

### Phase 1: Enum Tagging Support — **COMPLETE**
All four serde tagging modes work via `specta_serde::apply()`.

### Phase 2: Reference and Identity Support — **MOSTLY COMPLETE**
- ✅ `$id` on root schema
- ✅ `$id` per file in Files layout
- ✅ Files layout references fixed (relative + absolute)
- ✅ External `$ref` support
- ❌ `$anchor` support (see backlog)

### Phase 3: Struct Field Improvements — **MOSTLY COMPLETE**
- ✅ `allOf` for flattened fields
- ✅ `additionalProperties: false` (when no flatten)
- ✅ `description` on individual properties
- ❌ `default` values (requires upstream specta changes)

### Phase 4: Validation Keywords — **PARTIALLY COMPLETE**
- ✅ `oneOf` option
- ✅ `enum` array form
- ❌ `pattern` support (requires upstream specta changes)
- ❌ `minItems`/`maxItems` for Vec (requires upstream specta changes)

### Phase 5: Reference and Generic Handling — **COMPLETE**
- ✅ Generic reference resolution
- ✅ Circular reference support
- ✅ Import `const` handling improved

### Phase 6: Output Quality — **COMPLETE**
- ✅ `title` on definitions
- ✅ `#![allow(warnings)]` removed
- ✅ Comprehensive snapshot tests
