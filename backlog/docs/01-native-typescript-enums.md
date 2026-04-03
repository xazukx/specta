# Native TypeScript Enum Output

## Status: Not Currently Supported

Specta currently outputs all Rust enums as **TypeScript union types**, not native TypeScript `enum` declarations. There is no configuration option to switch output format.

### Current Behavior

```rust
#[derive(Type, Serialize)]
enum Status {
    Active,
    Inactive,
    Pending,
}
```

Produces:

```typescript
export type Status = "Active" | "Inactive" | "Pending";
```

### Desired Behavior

```rust
#[derive(Type, Serialize)]
#[specta(ts_enum)]
enum Status {
    Active,
    Inactive,
    Pending,
}
```

Produces:

```typescript
export enum Status {
    Active = "Active",
    Inactive = "Inactive",
    Pending = "Pending",
}
```

---

## Implementation Plan

### Background

The TypeScript exporter renders enums through `enum_datatype()` in `specta-typescript/src/legacy.rs:693`. Unit-variant enums (detected by `Enum::is_string_enum()` in `specta/src/datatype/enum.rs`) are currently rendered as string literal unions. Native TypeScript `enum` output would only make sense for these "string enums" — enums where every variant is a unit variant (no fields).

The trigger for native enum output is a per-enum opt-in attribute `#[specta(ts_enum)]`, not a global exporter setting. This keeps the default behavior unchanged and lets users choose native enums only where they need them.

### Step 1: Parse `#[specta(ts_enum)]` in the Derive Macro

Add `ts_enum` to the `ContainerAttr` struct in `specta-macros/src/type/attr/container.rs`:

```rust
#[derive(Default)]
pub struct ContainerAttr {
    // ... existing fields ...
    pub ts_enum: bool,
}
```

Parse it in `ContainerAttr::from_attrs()` alongside the existing attributes:

```rust
if let Some(attr) = attrs.extract("specta", "ts_enum") {
    result.ts_enum = attr.parse_bool().unwrap_or(true);
}
```

This follows the exact same pattern as `inline`, `transparent`, and `collect`.

### Step 2: Store in Runtime Attributes

In the macro codegen (in `specta-macros/src/type/mod.rs` or `specta-macros/src/type/enum.rs`), when `ts_enum` is true, insert it into the enum's runtime `Attributes` so it's available at export time:

```rust
// In the generated code that builds container_runtime_attrs
if container_attrs.ts_enum {
    // Insert into the Attributes that get applied to the Enum
    attrs.insert("specta:ts_enum", true);
}
```

The `Attributes` type in `specta/src/datatype/attributes.rs` is already a generic `HashMap<Cow<'static, str>, Arc<dyn DynAttributeValue>>` that supports arbitrary typed values. No changes needed to the Attributes type itself.

At runtime, the exporter reads it via:
```rust
let is_ts_enum = e.attributes().get_named_as::<bool>("specta:ts_enum").copied().unwrap_or(false);
```

### Step 3: Modify `enum_datatype()` to Render Native Enums

The existing `enum_datatype()` in `specta-typescript/src/legacy.rs:693` follows this flow:

```
1. Filter out skipped variants
2. Analyze discriminator (for tagged enums)
3. For each variant, call enum_variant_datatype() → get rendered string
4. Collect into EnumVariantOutput { value, strict_keys }
5. Strictify (add `?: never` for non-discriminated unions)
6. Wrap each value with inner_comments() for docs/deprecation
7. Deduplicate and join with " | "
8. Write to output string
```

For `#[specta(ts_enum)]`, we modify this flow **after** the filtered_variants collection (step 1) and **instead of** steps 2–8. The key insight is: we reuse the same filtered variant iteration and the `inner_comments()` call for docs, but change what each variant renders to and how they're joined.

Here's how `enum_datatype()` changes:

```rust
pub(crate) fn enum_datatype(
    ctx: ExportContext,
    e: &Enum,
    types: &Types,
    s: &mut String,
    prefix: &str,
    generics: &[(GenericReference, DataType)],
) -> Result<()> {
    if e.variants().is_empty() {
        return Ok(write!(s, "{NEVER}")?);
    }

    let filtered_variants = e
        .variants()
        .iter()
        .filter(|(_, variant)| !variant.skip())
        .collect::<Vec<_>>();

    // --- NEW: Check for #[specta(ts_enum)] ---
    let is_ts_enum = e.attributes()
        .get_named_as::<bool>("specta:ts_enum")
        .copied()
        .unwrap_or(false);

    if is_ts_enum && e.is_string_enum() {
        return render_native_ts_enum(ctx, &filtered_variants, s, prefix);
    }
    // --- END NEW ---

    // ... rest of existing function unchanged ...
}
```

The `render_native_ts_enum` function reuses the same variant collection and `inner_comments()` for doc/deprecation support:

```rust
fn render_native_ts_enum(
    ctx: ExportContext,
    filtered_variants: &[&(Cow<'static, str>, Variant)],
    s: &mut String,
    prefix: &str,
) -> Result<()> {
    // The inner prefix for enum members (one indent deeper than the enum itself)
    let member_prefix = format!("{prefix}\t");

    let mut members = Vec::with_capacity(filtered_variants.len());

    for (variant_name, variant) in filtered_variants.iter() {
        // The serialized value — sanitise_key with force_string=true gives us "VariantName"
        let serialized_value = sanitise_key(variant_name.clone(), true);
        // The member key — use the serialized name (what serde produces) as both key and value
        // so `#[serde(rename_all = "snake_case")]` is already applied by this point
        let member = format!("{} = {}", serialized_value, serialized_value);

        // Reuse inner_comments() for docs and deprecation, just like the union path does
        members.push(inner_comments(
            variant.deprecated(),
            variant.docs(),
            member,
            true,
            &member_prefix,
            !ctx.cfg.jsdoc,
        ));
    }

    // Deduplicate (same as the union path)
    let mut seen = BTreeSet::new();
    members.retain(|m| seen.insert(m.clone()));

    if members.is_empty() {
        s.push_str(NEVER);
    } else {
        // Join with ",\n" and wrap in braces — this is the native enum body
        // The caller (export_single_internal) will prepend "export enum Name"
        // instead of "export type Name = "
        s.push_str("{\n");
        s.push_str(&members.join(",\n"));
        s.push('\n');
        s.push_str(prefix);
        s.push('}');
    }

    Ok(())
}
```

**How variant names work with serde renames**: By this point in the pipeline, `specta_serde::apply()` has already applied `rename_all` and `rename` transformations to the variant names in the `Enum` datatype. So `variant_name` is already the serialized name (e.g., `"active_user"` for `ActiveUser` with `rename_all = "snake_case"`). The `sanitise_key(name, true)` call wraps it in quotes, giving us `"active_user"` — used for both the key and value of the enum member.

### Step 4: Adjust `export_single_internal()` for Native Enums

In `specta-typescript/src/primitives.rs:86`, the function currently always writes `export type Name = ...;`. For native enums, it needs to write `export enum Name { ... }` instead.

The `enum_datatype()` function writes directly into the output string `s`. We need to signal to the caller that this is a native enum so it uses a different wrapper. The cleanest approach: check the attribute on the `NamedDataType`'s inner `DataType` before writing the prefix.

```rust
fn export_single_internal(
    s: &mut String,
    exporter: &Exporter,
    types: &Types,
    ndt: &NamedDataType,
    indent: &str,
) -> Result<(), Error> {
    // ... existing JSDoc early return ...
    // ... existing generics computation ...
    // ... existing name computation ...
    // ... existing comments output ...

    // NEW: Detect if this is a native TS enum
    let is_native_enum = matches!(ndt.ty(), DataType::Enum(e)
        if e.is_string_enum()
        && e.attributes().get_named_as::<bool>("specta:ts_enum").copied().unwrap_or(false)
    );

    s.push_str(indent);
    if is_native_enum {
        s.push_str("export enum ");
        s.push_str(&name);
        // No generics for native enums (string enums can't be generic)
        s.push(' ');

        let _generic_scope = push_generic_scope(ndt.generics());
        datatype(s, exporter, types, ndt.ty(), /* ... */)?;
        s.push('\n');
    } else {
        s.push_str("export type ");
        s.push_str(&name);
        for part in generics {
            s.push_str(part);
        }
        s.push_str(" = ");

        let _generic_scope = push_generic_scope(ndt.generics());
        datatype(s, exporter, types, ndt.ty(), /* ... */)?;
        s.push_str(";\n");
    }

    Ok(())
}
```

The native enum path writes `export enum Name ` (note the space, not ` = `), then `enum_datatype()` writes `{\n  ...\n}`, then we close with `\n` instead of `;\n`.

### Step 5: Handle Imports

Native TypeScript enums are values, not just types. When another type references a native enum:
- In type position, it works as-is (TypeScript enums are also types)
- In `import type` statements, this is fine for type-only usage
- If runtime usage is needed (e.g., in Zod schemas), the import would need to be a value import (`import { Status }` instead of `import type { Status }`)

For the `Layout::Files` mode, the import generation in `module_import_statement()` may need adjustment to use value imports for native enums.

### Step 6: Constraints and Validation

The `#[specta(ts_enum)]` attribute should produce an error at the serde validation stage (or a clear runtime error) if:
- The enum has non-unit variants (struct/tuple variants can't be native TS enum members)
- The enum is generic (native TS enums can't be generic)

If `is_string_enum()` is false when `ts_enum` is set, emit a descriptive error rather than silently falling back — the user explicitly asked for native enum output, so they should know it can't be done.

JSDoc mode: Native `enum` syntax is not valid in JSDoc. If `ts_enum` is set and JSDoc mode is active, fall back to `@enum` syntax.

### Step 7: Serde Rename Support

Serde renames are already applied to variant names by `specta_serde::apply()` before the TypeScript exporter runs. So:

```rust
#[derive(Type, Serialize)]
#[specta(ts_enum)]
#[serde(rename_all = "snake_case")]
enum Status {
    ActiveUser,
    InactiveUser,
}
```

After `specta_serde::apply()`, the variant names in the `Enum` datatype are already `"active_user"` and `"inactive_user"`. The exporter uses these directly:

```typescript
export enum Status {
    "active_user" = "active_user",
    "inactive_user" = "inactive_user",
}
```

Note: TypeScript enum keys containing non-identifier characters (like hyphens from `kebab-case`) must be quoted. The `sanitise_key` function already handles this — it quotes keys that aren't valid identifiers.

---

## Usage (After Implementation)

```rust
use serde::Serialize;
use specta::Type;

#[derive(Type, Serialize)]
#[specta(ts_enum)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Type, Serialize)]
#[specta(ts_enum)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}
```

```rust
let types = specta::Types::default()
    .register::<Direction>()
    .register::<LogLevel>();

let resolved = specta_serde::apply(types).unwrap();

specta_typescript::Typescript::default()
    .export_to("./bindings.ts", &resolved)
    .unwrap();
```

Output:

```typescript
// This file has been generated by Specta. Do not edit this file manually.

export enum Direction {
    Up = "Up",
    Down = "Down",
    Left = "Left",
    Right = "Right",
}

export enum LogLevel {
    DEBUG = "DEBUG",
    INFO = "INFO",
    WARN = "WARN",
    ERROR = "ERROR",
}
```

Enums **without** `#[specta(ts_enum)]` continue to use the existing union type output:

```rust
#[derive(Type, Serialize)]
enum Color {
    Red,
    Green,
    Blue,
}
// Output: export type Color = "Red" | "Green" | "Blue";
```

---

## Files to Modify

| File | Change |
|------|--------|
| `specta-macros/src/type/attr/container.rs` | Add `ts_enum: bool` field to `ContainerAttr`, parse `#[specta(ts_enum)]` |
| `specta-macros/src/type/mod.rs` or `specta-macros/src/type/enum.rs` | Insert `"specta:ts_enum"` into runtime Attributes when flag is set |
| `specta-typescript/src/legacy.rs` | Add `render_native_ts_enum()` function; check attribute in `enum_datatype()` to branch to it |
| `specta-typescript/src/primitives.rs` | Modify `export_single_internal()` to emit `export enum Name { ... }` instead of `export type Name = ...;` when attribute is present |
| `tests/tests/typescript.rs` | Add snapshot tests for `#[specta(ts_enum)]` output |
