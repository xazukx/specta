# TypeScript: Optional Syntax and Cross-File Imports

## Status: Research / Backlog

---

## Rust `Option<T>` to TypeScript Optional Syntax

### Current Behavior

Specta distinguishes between two separate concepts when exporting to TypeScript:

- **Nullable type** (`DataType::Nullable`): renders as `T | null`
- **Optional field** (`field.optional()`): renders as `field?: T`

An `Option<String>` field in Rust does **not** automatically become `field?: string`. Instead, specta uses serde semantics to decide optionality.

### How It Works

**Step 1 -- Nullable wrapping**
`Option<T>` is always represented as `DataType::Nullable(T)`, which renders as `T | null` in the TypeScript output. This happens in `specta-typescript/src/primitives.rs` (around line 1013):

```typescript
// Rust: Option<String>
// TS output:
field: string | null
```

**Step 2 -- Optional field syntax**
The `?:` syntax is added when `field.optional()` returns `true`. This is determined by the serde processor in `specta-serde/src/lib.rs` (around line 1985). A field becomes optional when:

- The field has `#[serde(default)]`
- The container (struct) has `#[serde(default)]`
- The field has `#[serde(skip_deserializing)]`
- The field has `#[specta(optional)]`

The field name gets `?` appended in `specta-typescript/src/legacy.rs` (`object_field_to_ts`, around line 850):

```rust
let (key, ty) = match field.optional() {
    true => (format!("{field_name_safe}?").into(), ty),
    false => (field_name_safe, ty),
};
```

### Three Output Patterns

| Rust Definition | TypeScript Output | Meaning |
|---|---|---|
| `field: Option<String>` | `field: string \| null` | Must be present, value can be null |
| `#[serde(default)] field: String` | `field?: string` | Can be omitted, but if present must be a string |
| `#[specta(optional)] field: Option<String>` | `field?: string \| null` | Can be omitted AND value can be null |

### Examples from Test Snapshots

```rust
// Simple Option -- NOT optional, IS nullable
struct SimpleStruct { e: Option<String> }
// => e: string | null

// Field-level default -- IS optional, NOT nullable
struct FieldDefault { #[serde(default)] enabled: bool }
// => enabled?: boolean

// Container default -- ALL fields optional
#[serde(default)]
struct ContainerDefault { value: String, flag: bool }
// => value?: string, flag?: boolean

// Explicit optional + nullable
enum OptionalInEnum { C { #[specta(optional)] a: Option<String> } }
// => a?: string | null
```

### Summary

Specta correctly supports the `name?: string` optional syntax. It is **not** driven by `Option<T>` alone -- it follows serde deserialization semantics. `Option<T>` contributes nullability (`| null`), while `#[serde(default)]` or `#[specta(optional)]` contribute optionality (`?:`). This is the correct behavior because it matches how serde actually deserializes JSON.

---

## Cross-File Imports and File Path Resolution

### Layout Modes

The `Layout` enum (`specta-typescript/src/exporter.rs:18-31`) defines four export strategies:

```rust
pub enum Layout {
    Namespaces,           // TypeScript namespaces per Rust module
    Files,                // Dedicated file per Rust module
    ModulePrefixedName,   // Module path baked into type name, single file
    FlatFile,             // (default) All types in one file, no module structure
}
```

### How `Files` Layout Determines File Paths

Rust module paths (`::` separated) map directly to file system directories:

| Rust Module Path | Generated File |
|---|---|
| `test::layouts::testing::testing2` | `layouts/testing/testing2.ts` |
| `test::layouts::testing` | `layouts/testing.ts` |
| `test::layouts` | `layouts.ts` |
| (empty / root) | `index.ts` |

The crate name prefix is stripped. The extension is `.ts` (or `.js` in JSDoc mode).

The module graph is built by `build_module_graph()` (exporter.rs lines 557-591), which splits all `NamedDataType` module paths on `::` and creates a hierarchical tree of `Module` nodes.

### How Imports Are Generated

When a type in one file references a type from another module, specta generates import statements automatically.

**Reference tracking mechanism** (`specta-typescript/src/references.rs`):
1. During rendering, `collect_references()` wraps the render call and captures all `NamedReference` instances via thread-local storage
2. Each time a type is referenced during rendering, `track_nr()` records it
3. After rendering, the set of referenced types is filtered to those in different modules
4. Import statements are generated for each unique foreign module

**Import statement format:**

TypeScript mode:
```typescript
import type * as test$layouts$testing from "./layouts/testing";
```

JSDoc mode:
```javascript
/**
 * @typedef {import("./layouts/testing")} test$layouts$testing
 */
```

**Module alias convention** (exporter.rs lines 856-862):
- `::` separators replaced with `$`
- `test::layouts::testing` becomes `test$layouts$testing`
- Empty module path becomes `$root`

### Relative Path Calculation

The function `module_import_path()` (exporter.rs lines 910-944) computes relative paths between two module files:

**Algorithm:**
1. Convert both module paths to file segments (empty path becomes `["index"]`)
2. Extract directory segments from the source (all but last segment)
3. Find the longest shared prefix between source directory and target path
4. Build the relative path: `..` for each unshared source segment, then the remaining target segments
5. Ensure the path starts with `.` or `..`

**Examples:**

| From Module | To Module | Generated Import Path |
|---|---|---|
| `test::layouts` | `test::layouts::testing` | `./layouts/testing` |
| `test::layouts::testing` | `test::layouts::testing::testing2` | `./testing/testing2` |
| `a::b::c` | `a::d` | `../d` |
| `test::layouts::testing` | `test::layouts` | `../layouts` |

### How Cross-File Type References Render

When a type from module A references a type from module B in `Files` layout, the reference renders as `alias.TypeName` (primitives.rs lines 2032-2043):

```rust
if ndt.module_path() == &current_module_path {
    ndt.name().clone()                          // Same module: plain name
} else {
    let mut path = module_alias(ndt.module_path());
    path.push('.');
    path.push_str(ndt.name());
    Cow::Owned(path)                            // Different module: alias.TypeName
}
```

### Real Snapshot Example

From the test snapshots, a three-level module hierarchy produces:

**File: `layouts/testing/testing2.ts`**
```typescript
export type Testing = {
    c: string,
};
```

**File: `layouts/testing.ts`**
```typescript
import type * as test$layouts$testing$testing2 from "./testing/testing2";

export type Testing = {
    b: test$layouts$testing$testing2.Testing,
};
```

**File: `layouts.ts`**
```typescript
import type * as test$layouts$testing from "./layouts/testing";

export type Testing = {
    a: test$layouts$testing.Testing,
};
```

### Cross-Crate Type References

Cross-crate references work identically to in-crate references. The `NamedDataType::module_path()` includes the crate name as a prefix (e.g., `external_crate::module::Type`), so the same path resolution algorithm handles them transparently. The crate name becomes part of the directory structure.

### Other Layout Behaviors

| Layout | Type Name Format | Import Style |
|---|---|---|
| **FlatFile** (default) | `TypeName` | No imports (single file) |
| **ModulePrefixedName** | `test_layouts_Testing` | No imports (single file, module path in name) |
| **Namespaces** | `$s$.test.layouts.testing.Testing` | No imports (single file, nested TS namespaces) |
| **Files** | `test$layouts$testing.Testing` | `import type * as alias from "./relative/path"` |
