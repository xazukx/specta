# JSON Schema Exporter: Remaining Backlog

## Status: Backlog

Items that are not yet implemented in the JSON Schema exporter. Organized by feasibility.

---

## Feasible Without Upstream Changes

### `$anchor` Support (Phase 2)

**What:** The `$anchor` keyword (Draft 2019-09+) creates named reference targets independent of document structure. Types would get stable anchors like `{"$anchor": "User"}` and could be referenced as `{"$ref": "#User"}`.

**Why:** More stable references that don't break when document structure changes. Useful for schema composition.

**Difficulty:** Medium. Needs:
- Auto-generate anchors from type names, or support a `#[specta(anchor = "...")]` attribute
- New `$ref` generation mode to use `#anchor` syntax
- Only emit for Draft 2019-09+

### `$id` Per Definition (Phase 2)

**What:** Each entry in the definitions map gets its own `$id`, e.g. `"$id": "https://example.com/schemas/User"`.

**Why:** Enables individual definitions to be referenced by URI from other documents.

**Difficulty:** Easy. In `export_single_file`, add `"$id": format!("{}/{}", base_uri, name)` to each definition object. Requires `base_uri` to be set.

### Validate Output Against JSON Schema Meta-Schema (Phase 6)

**What:** Add integration tests that validate the exporter's output against the official JSON Schema meta-schemas (Draft 7, 2019-09, 2020-12).

**Why:** Ensures output is spec-compliant, catches regressions.

**Difficulty:** Medium. Add `jsonschema` crate as dev-dependency, write test that validates each schema version's output against the corresponding meta-schema.

### Test Interop With Popular Validators (Phase 7)

**What:** Generate schemas and validate sample data against them using `jsonschema` (Rust), ajv (JS), or similar.

**Why:** Ensures schemas actually validate correctly, not just that they look right structurally.

**Difficulty:** Medium. Testing infrastructure only.

### Improve Import Functionality (Phase 7)

**What:** The import module (`import.rs`) doesn't handle `allOf` composition, proper nested unions, or complex `$ref` resolution.

**Difficulty:** Medium. Each missing construct is a small addition but there are many. Could be done incrementally.

### OpenAPI 3.x Compatibility Mode (Phase 7)

**What:** OpenAPI 3.0 uses a JSON Schema subset with differences: `nullable` instead of `anyOf` with null, limited `$ref` usage, `discriminator` keyword for tagged unions.

**Why:** Many users generate schemas for OpenAPI documentation.

**Difficulty:** Hard. Significant design work — either a separate output mode or a set of compatibility flags. Would need:
- `nullable: true` instead of `anyOf [T, null]`
- `discriminator` keyword for internally-tagged enums
- Restricted `$ref` usage patterns

---

## Blocked on Upstream Specta Changes

These items require new data or attributes in the specta core data model.

### `default` Values (Phase 3)

**What:** Emit `"default"` keyword for fields with `#[serde(default)]`.

**Why it's blocked:** Specta's `Field` struct doesn't carry default value information. The `#[serde(default)]` attribute tells serde to use `Default::default()`, but the concrete value isn't available at the type level.

**What's needed:** A new specta attribute like `#[specta(default = "value")]` or compile-time default value capture.

### `pattern` Support (Phase 4)

**What:** Emit `"pattern"` keyword for string types with regex validation.

**Why it's blocked:** Specta's data model doesn't carry regex metadata.

**What's needed:** A new attribute like `#[specta(pattern = "^[a-z]+$")]`.

### `minItems` / `maxItems` for Vec (Phase 4)

**What:** Emit length constraints on array types.

**Why it's blocked:** Specta doesn't carry length constraints on collections (only on fixed-length arrays).

**What's needed:** Attributes like `#[specta(min_items = 1, max_items = 100)]`.

### `examples` Support (Phase 6)

**What:** Emit `"examples"` keyword with sample values.

**Why it's blocked:** Specta doesn't carry example values.

**What's needed:** A new attribute like `#[specta(example = r#"{"name": "Alice"}"#)]`.

### `minimum` / `maximum` / `exclusiveMinimum` / `exclusiveMaximum` for Numbers

**What:** Emit numeric range constraints beyond what's implied by the Rust type (e.g. i8 bounds).

**Why it's blocked:** No way to attach custom range constraints to fields.

**What's needed:** Attributes like `#[specta(minimum = 0, maximum = 100)]`.

### `minLength` / `maxLength` for Strings

**What:** Emit string length constraints.

**Why it's blocked:** Same as above — no metadata on the field.

**What's needed:** Attributes like `#[specta(min_length = 1)]`.
