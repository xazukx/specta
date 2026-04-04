# JSON Schema Exporter: Current State and Implementation Plan

## Status: Research / Backlog

---

## Current State

The specta-jsonschema exporter is **functional but incomplete**. It supports core JSON Schema constructs and three draft versions, but has notable gaps in enum tagging support and reference handling. The crate has `#![allow(warnings)]` at the top, indicating the author considers it a work in progress.

### Schema Versions Supported

| Version | `$schema` URI | Definitions Key |
|---|---|---|
| **Draft 7** (default) | `http://json-schema.org/draft-07/schema#` | `definitions` |
| **Draft 2019-09** | `https://json-schema.org/draft/2019-09/schema` | `$defs` |
| **Draft 2020-12** | `https://json-schema.org/draft/2020-12/schema` | `$defs` |

### Currently Supported Constructs

#### Primitives (fully supported)

| Rust Type | JSON Schema Output |
|---|---|
| `bool` | `{"type": "boolean"}` |
| `String` | `{"type": "string"}` |
| `char` | `{"type": "string", "minLength": 1, "maxLength": 1}` |
| `i8` | `{"type": "integer", "minimum": -128, "maximum": 127}` |
| `i16` | `{"type": "integer", "minimum": -32768, "maximum": 32767}` |
| `i32` | `{"type": "integer", "format": "int32"}` |
| `i64` | `{"type": "integer", "format": "int64"}` |
| `u32` | `{"type": "integer", "minimum": 0, "format": "uint32"}` |
| `f32` | `{"type": "number", "format": "float"}` |
| `f64` | `{"type": "number", "format": "double"}` |

#### Collections

| Rust Type | JSON Schema Output |
|---|---|
| `Vec<T>` | `{"type": "array", "items": <T>}` |
| `[T; N]` | `{"type": "array", "prefixItems": [...], "minItems": N, "maxItems": N, "items": false}` |
| `(T1, T2)` | `{"type": "array", "prefixItems": [...], "minItems": N, "maxItems": N, "items": false}` |
| `HashMap<K, V>` | `{"type": "object", "additionalProperties": <V>}` |
| `()` | `{"type": "null"}` |

#### Structs

| Rust Type | JSON Schema Output |
|---|---|
| Unit struct | `{"type": "null"}` |
| Tuple struct | `{"type": "array", "prefixItems": [...]}` |
| Named struct | `{"type": "object", "properties": {...}, "required": [...]}` |

#### Enums (partially supported -- external tagging only)

| Pattern | JSON Schema Output |
|---|---|
| Unit variant | `{"const": "VariantName"}` |
| Tuple variant | `{"type": "object", "required": ["Name"], "properties": {"Name": ...}, "additionalProperties": false}` |
| Named variant | `{"type": "object", "required": ["Name"], "properties": {"Name": ...}, "additionalProperties": false}` |
| Multiple variants | `{"anyOf": [...]}` |

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
```

### Test Coverage

Tests exist but are minimal:
- `test_basic_export` -- basic struct + enum export
- `test_schema_version` -- URI verification
- `test_primitives` -- primitive type mapping
- `test_nullable` -- Option handling
- `test_enum` -- simple enum variants

No snapshot tests, no complex nested types, no tagging variation tests.

---

## `$id`, `$anchor`, `$ref`, and Definitions Handling

### `$ref` -- How References Work

**Supported.** References are generated for named types using fragment-only JSON pointers.

The code in `specta-jsonschema/src/primitives.rs` (lines 90-95) generates references:

```rust
let defs_key = js.schema_version.definitions_key();
if let Some(referenced_ndt) = r.get(types) {
    Ok(json!({
        "$ref": format!("#/{}/{}", defs_key, referenced_ndt.name())
    }))
}
```

**Format by draft version:**
- Draft 7: `{"$ref": "#/definitions/TypeName"}`
- Draft 2019-09: `{"$ref": "#/$defs/TypeName"}`
- Draft 2020-12: `{"$ref": "#/$defs/TypeName"}`

All references are **fragment-only** (same-document with `#` prefix). No external URI references are supported.

### `definitions` / `$defs` -- How the Definitions Map is Built

**Supported.** The definitions key adapts automatically based on `SchemaVersion`.

The code in `specta-jsonschema/src/json_schema.rs` (lines 91-106) builds definitions:

```rust
fn export_single_file(&self, types: &Types) -> Result<Value, Error> {
    let mut definitions = BTreeMap::new();
    for ndt in types.into_sorted_iter() {
        let schema = primitives::export(self, types, &ndt)?;
        let name = ndt.name().to_string();
        definitions.insert(name, schema);
    }
    let defs_key = self.schema_version.definitions_key();
    let mut root = serde_json::json!({
        "$schema": self.schema_version.uri(),
        defs_key: definitions,
    });
    // ...
}
```

**SingleFile layout** produces:
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "User": { "type": "object", "properties": { ... } },
    "Role": { "anyOf": [ ... ] }
  }
}
```

**Files layout** produces one `.schema.json` per type, with each file containing just that type's schema and a `$schema` keyword -- but **no definitions section** in individual files.

### `$id` -- Not Supported

**Not implemented.** The codebase contains no handling of `$id` anywhere:

- No `$id` is assigned to the root schema document
- No `$id` is assigned to individual definition entries
- No `$id` is assigned to individual files in `Files` layout
- No configuration option exists for a base URI

This means:
- Schemas can only be referenced via fragment (`#/definitions/...`), not by URI
- In `Files` layout, `$ref` pointers to `#/$defs/TypeName` are **broken** because the definitions don't exist in the individual files
- No way to compose schemas across multiple documents using URI references

### `$anchor` -- Not Supported

**Not implemented.** The `$anchor` keyword (introduced in Draft 2019-09) is not generated or supported in any way. This means:
- Types can only be referenced by their JSON Pointer path, not by anchor name
- No way to create stable reference targets independent of document structure

### Reference Types Summary

| JSON Schema Keyword | Supported? | Notes |
|---|---|---|
| `$ref` (fragment) | Yes | `#/definitions/Name` or `#/$defs/Name` |
| `$ref` (external URI) | No | No external references |
| `$id` (root) | No | No base URI |
| `$id` (per definition) | No | Definitions have no identity |
| `$anchor` | No | Not implemented |
| `definitions` (Draft 7) | Yes | Automatic based on schema version |
| `$defs` (2019-09+) | Yes | Automatic based on schema version |

### Reference Customization

**Very limited.** There is no way to:
- Set a custom base URI for `$ref` targets
- Change the `$ref` prefix format
- Use external references instead of fragments
- Add `$id` to definitions or files

The only configurable aspect is the definitions key (`definitions` vs `$defs`), which changes automatically based on `SchemaVersion`.

### Opaque and Generic References

- **Named references**: Work correctly via `$ref`
- **Generic references**: Produce empty schema `{}` (accepts anything) -- a placeholder
- **Opaque references**: Produce an error: "Opaque references are not supported by JSON Schema exporter"

---

## Known Limitations

1. **Enum tagging** -- Only **external tagging** is supported. No support for:
   - Internal tagging (`#[serde(tag = "type")]`)
   - Adjacent tagging (`#[serde(tag = "t", content = "c")]`)
   - Untagged (`#[serde(untagged)]`)
2. **No `$id` or `$anchor`** -- Cannot compose schemas across documents or create stable reference targets
3. **Files layout references are broken** -- `$ref` points to local `#/$defs/...` paths that don't exist in individual files
4. **Generic references** -- Produce empty schema `{}` instead of proper resolution
5. **No `allOf` support** -- Flattened fields not properly represented
6. **No `oneOf`** -- Uses `anyOf` exclusively (less strict validation for discriminated unions)
7. **No `title`/`description` on definitions** -- Only at root level
8. **No `examples` or `default`** generation
9. **No `pattern` or `enum` (array form)** generation
10. **Suppressed warnings** -- `#![allow(warnings)]` indicates known incomplete state
11. **Import `const` handling** -- Maps `const` values to string type (not true literal support)

---

## Implementation Plan: Complete JSON Schema Support

### Phase 1: Enum Tagging Support (Priority: Critical)

This is the largest gap in the current implementation.

1. **Implement internal tagging** (`#[serde(tag = "type")]`)
   - Unit variants: `{"type": "object", "required": ["type"], "properties": {"type": {"const": "VariantName"}}}`
   - Named variants: merge tag field into the variant object's properties
   - Tuple variants: not supported by serde with internal tagging (error)

2. **Implement adjacent tagging** (`#[serde(tag = "t", content = "c")]`)
   - All variants: `{"type": "object", "required": ["t", "c"], "properties": {"t": {"const": "VariantName"}, "c": <content schema>}}`

3. **Implement untagged enums** (`#[serde(untagged)]`)
   - Use `anyOf` with just the inner type schemas (no wrapping object)
   - Consider using `oneOf` for stricter validation

4. **Add enum representation detection** -- read serde attributes to determine which tagging style to use (the serde processor already provides this information via specta-serde)

### Phase 2: Reference and Identity Support (Priority: High)

5. **Add `$id` to root schema** -- configurable base URI (e.g., `https://example.com/schemas/`)
6. **Add `$id` to individual definitions** -- compose base URI + type name (e.g., `https://example.com/schemas/User`)
7. **Fix Files layout references** -- when emitting separate files, use either:
   - Relative file URI references (`{"$ref": "./User.schema.json"}`)
   - Absolute URI references if base URI is configured
8. **Add `$anchor` support** -- allow types to define stable anchors independent of document structure
9. **Support external `$ref`** -- allow referencing schemas in other files/URIs

### Phase 3: Struct Field Improvements (Priority: High)

10. **Implement `allOf` for flattened fields** -- when a struct has `#[serde(flatten)]` fields, combine schemas using `allOf`
11. **Add `additionalProperties: false`** option for strict object schemas
12. **Support `description` on individual properties** -- if specta carries doc comments, propagate them to JSON Schema `description`
13. **Support `default` values** -- if fields have `#[serde(default)]` with known values, emit `"default"` in the schema

### Phase 4: Validation Keywords (Priority: Medium)

14. **Add `oneOf` as an option** -- for discriminated unions (internally tagged enums), `oneOf` is more semantically correct than `anyOf`
15. **Add `enum` (array form)** -- for simple string/number enums, emit `{"enum": ["A", "B", "C"]}` instead of `{"anyOf": [{"const": "A"}, ...]}`
16. **Add `pattern` support** -- if specta types carry regex metadata
17. **Add `minItems`/`maxItems` for Vec** -- if specta types carry length constraints

### Phase 5: Reference and Generic Handling (Priority: Medium)

18. **Resolve generic references properly** -- instead of empty schema `{}`, either inline the resolved type or use `$ref` with the concrete type name
19. **Support circular references** -- ensure recursive types work correctly with `$ref`
20. **Improve import `const` handling** -- map `const` to proper literal types instead of string

### Phase 6: Output Quality (Priority: Low)

21. **Add `title` to definitions** -- each definition should carry its Rust type name as `title`
22. **Add `examples`** -- if specta types carry example values
23. **Remove `#![allow(warnings)]`** -- fix all warnings once implementation is more complete
24. **Add comprehensive snapshot tests** -- cover all tagging styles, nested types, generics, and edge cases
25. **Validate output against JSON Schema meta-schema** -- ensure generated schemas are valid JSON Schema documents

### Phase 7: Compatibility and Ecosystem (Priority: Low)

26. **Test interop with popular validators** -- ensure output works with ajv, jsonschema (Python), and other popular validators
27. **Add OpenAPI 3.x compatibility mode** -- OpenAPI uses a subset/superset of JSON Schema; consider an option to emit compatible output
28. **Improve import functionality** -- support `allOf`, proper literal types, and complex union patterns
