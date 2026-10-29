---
id: TASK-005
title: Binary serializer in rust and typescript
status: To Do
assignee: []
created_date: '2026-10-28 19:03'
labels: [feature, typescript, binary]
dependencies: []
---

# Binary serializer in Rust and TypeScript

## Goal

Add a compact, self-describing binary wire format called "Specta Binary" (`SPB`). It is designed so that the TypeScript side can decode it cheaply in the browser.

It works the way `serde_json` and `JSON.parse` do. **There is no schema and no code generation.**

- **Rust side:** a `serde` serializer and deserializer in a new crate, `specta-binary`. Any `Serialize`/`Deserialize` type works without an extra derive.
- **TypeScript side:** a hand-written, dependency-free runtime with two functions:
  - `encode(value: unknown): Uint8Array`
  - `decode<T = unknown>(bytes: Uint8Array): T`

  Users decode straight into the types `specta-typescript` already generates, for example `decode<User>(bytes)`.
- **`specta-typescript` is not changed.** It keeps outputting TypeScript types only.
- **Every value is preceded by a one-byte type tag** (`ValueType`). Strings, byte arrays, arrays and objects each have a "short" variant with a 1-byte length.
- **Optional key table.** A document can start with a table of property names. Each name's id is its position in the table, so `1Name2Email3Address` becomes `[0]"name" [1]"email" [2]"address"`. Object keys then refer to a name by its id instead of repeating the string.
  - Only object keys use the table. No other value does.
  - The table is chosen per serialize call. The reader detects it from the header, so decoding needs no configuration.

### Guiding principle

> A decoded value has the same JS shape as `JSON.parse(serde_json::to_string(&value))`. The format adds typed numbers, raw bytes, short lengths and interned keys on top of that.

Because the format is the serde/JSON data model, all serde attributes work for free: `rename`, `tag`, `untagged`, `flatten`, `skip_serializing_if`, `with`, and the rest. Serde handles them in Rust. TypeScript just sees the same objects, arrays and strings that JSON would produce.

---

## 1. Wire format specification

Write sections 1 and 2 out as `specta-binary/SPEC.md`. That file is the source of truth for both implementations.

### 1.1 Conventions

- Fixed-width numbers are **little-endian**. This matches `DataView` with `littleEndian = true`.
- `varint` means an unsigned LEB128 integer, capped at 32 bits (at most 5 bytes).
  - Readers must reject varints that are longer than 5 bytes or overflow 32 bits.
- `u8len` means a single unsigned byte, 0–255. It is read directly, with **no varint decoding**.
- Strings are UTF-8 and are not NUL-terminated.

### 1.2 Value type tags

```rust
#[repr(u8)]
pub enum ValueType {
    Null            = 0,   // 0x00
    Bool            = 5,   // 0x05
    Int32           = 10,  // 0x0A
    Int64           = 15,  // 0x0F
    Float32         = 20,  // 0x14
    Float64         = 25,  // 0x19
    StringUtf8      = 30,  // 0x1E
    ShortStringUtf8 = 35,  // 0x23
    ByteArray       = 40,  // 0x28
    ShortByteArray  = 45,  // 0x2D
    Array           = 50,  // 0x32
    ShortArray      = 55,  // 0x37
    Object          = 60,  // 0x3C
    ShortObject     = 65,  // 0x41
}
```

| Tag               | Payload after the tag                                          |
|-------------------|----------------------------------------------------------------|
| `Null`            | nothing                                                        |
| `Bool`            | 1 byte: `0x00` = false, `0x01` = true. Any other byte is an error |
| `Int32`           | 4 bytes, `i32` LE                                              |
| `Int64`           | 8 bytes, `i64` LE                                              |
| `Float32`         | 4 bytes, IEEE-754 `f32` LE                                     |
| `Float64`         | 8 bytes, IEEE-754 `f64` LE                                     |
| `StringUtf8`      | `varint byteLen`, then `byteLen` UTF-8 bytes                   |
| `ShortStringUtf8` | `u8len byteLen`, then `byteLen` UTF-8 bytes                    |
| `ByteArray`       | `varint byteLen`, then `byteLen` raw bytes                     |
| `ShortByteArray`  | `u8len byteLen`, then `byteLen` raw bytes                      |
| `Array`           | `varint count`, then `count` tagged values                     |
| `ShortArray`      | `u8len count`, then `count` tagged values                      |
| `Object`          | `varint count`, then `count` × (`key`, tagged value). See 1.4  |
| `ShortObject`     | `u8len count`, then `count` × (`key`, tagged value). See 1.4   |

**Short-form rules:**

- **Writers** must use the short variant whenever the length or count is ≤ 255, and the long variant otherwise. This keeps the output deterministic.
- **Readers** must accept both variants for any length. A long variant holding a length ≤ 255 is still valid input.

Readers must reject unknown tags with an error that includes the byte offset. The gaps between tag values are left for future tags.

### 1.3 Document layout

```
+----------+----------+---------+-------+----------------------+-------------------+
| 'S' 0x53 | 'B' 0x42 | version | flags | [key table, if flag] | root tagged value |
+----------+----------+---------+-------+----------------------+-------------------+
```

- `version` is `0x01`. Readers must reject versions they do not know.
- `flags` is a bitset:
  - bit 0, `HAS_KEY_TABLE`: a key table follows the header.
  - All other bits must be 0, and readers must reject a document if they are set.
- **Key table**, present only when `HAS_KEY_TABLE` is set:
  ```
  varint keyCount
  keyCount × ( varint byteLen, byteLen UTF-8 bytes )
  ```
  - A key's id is its zero-based position in the table. Ids are implicit, so they cost no bytes.
  - Ids are assigned in **first-use order** during a depth-first walk of the value, which keeps the output deterministic.
  - The table holds **only object keys**. Strings that are values are never interned.
- After the header (and table, if present) comes **exactly one** root value. Trailing bytes are an error.

### 1.4 Object keys

The `HAS_KEY_TABLE` flag decides how every key inside an `Object`/`ShortObject` is encoded, for the whole document:

| `HAS_KEY_TABLE` | Key encoding                                                         |
|-----------------|----------------------------------------------------------------------|
| set             | `varint keyId`. It must be < `keyCount`                              |
| clear           | `varint byteLen`, then `byteLen` UTF-8 bytes (the key written inline) |

Keys are never tagged, because they are always strings.

When the writer has the key table enabled:

- If the document contains **no objects**, it emits `flags = 0` and no table. Objects get a table; other documents don't.
- An empty table is never written.

### 1.5 Numbers

**Writers:**

- Integers use the **narrowest tag that holds the value losslessly**: `Int32` if it is in `i32` range, otherwise `Int64`.
- `u64` values above `i64::MAX` are a serialization error. This avoids a silent sign flip, since there is no schema to say the value is unsigned.
- `i128`/`u128` follow the same rule: write them if they fit in `i64`, otherwise return an error.
- Rust `f32` → `Float32`, Rust `f64` → `Float64`. Floats are never narrowed.
- The TS encoder only has `number`. It writes integer values (`Number.isInteger`) as `Int32`/`Int64` and everything else as `Float64`.
- `NaN` and `±Infinity` are allowed. Note that this is a deliberate difference from JSON.

**Readers:** with no schema, readers must be lenient. The Rust deserializer:

- accepts `Int32` or `Int64` for any integer target, and range-checks the value;
- accepts **any** numeric tag for a float target, because JS cannot tell `1.0` from `1`;
- converts `f64` to `f32` with `as`, the same way `serde_json` does;
- treats anything else as a type-mismatch error that includes the path and offset.

### 1.6 Limits (both readers)

These limits guard against hostile or corrupted input:

- Maximum nesting depth: 128 by default, configurable.
- Each length or count must be ≤ the number of bytes remaining. Check this before allocating.
- Each key id must be < `keyCount`.
- String bytes must be valid UTF-8. In TS, use `new TextDecoder("utf-8", { fatal: true })`.

### 1.7 Worked examples

```rust
#[derive(Serialize, Deserialize, Type)]
struct User { name: String, email: Option<String>, age: u8 }

User { name: "Ann".into(), email: None, age: 30 }
```

**With the key table** (the default):

```
53 42 01 01                 header: "SB", v1, HAS_KEY_TABLE
03                          3 keys
04 6E 61 6D 65              [0] "name"
05 65 6D 61 69 6C           [1] "email"
03 61 67 65                 [2] "age"
41 03                       ShortObject, 3 entries
00   23 03 41 6E 6E         key#0 -> ShortStringUtf8 "Ann"
01   00                     key#1 -> Null
02   0A 1E 00 00 00         key#2 -> Int32 30
```

**Without the key table:**

```
53 42 01 00                 header: "SB", v1, no flags
41 03                       ShortObject, 3 entries
04 6E 61 6D 65   23 03 41 6E 6E      "name"  -> ShortStringUtf8 "Ann"
05 65 6D 61 69 6C   00               "email" -> Null
03 61 67 65   0A 1E 00 00 00         "age"   -> Int32 30
```

**A document with no object**, for example `vec![1, 2]`:

```
53 42 01 00   37 02   0A 01 00 00 00   0A 02 00 00 00
```

Make all three of these golden snapshot tests (see section 5).

---

## 2. Mapping the serde data model to wire values

This is the same as `serde_json`. Any differences are called out.

| serde / Rust                                   | Wire                                    | `decode` result in TS                    |
|------------------------------------------------|-----------------------------------------|------------------------------------------|
| `bool`                                         | `Bool`                                  | `boolean`                                |
| any integer                                    | `Int32` / `Int64` (narrowest, see 1.5)  | `number`, or `bigint` (see 4.2)          |
| `f32` / `f64`                                  | `Float32` / `Float64`                   | `number`                                 |
| `char`, `str`, `String`                        | `(Short)StringUtf8`                     | `string`                                 |
| `serialize_bytes` (`serde_bytes`, `bytes::Bytes`) | `(Short)ByteArray`                   | `Uint8Array` (see 4.2)                   |
| `()`, unit struct, `None`                      | `Null`                                  | `null`                                   |
| `Some(x)`, newtype struct                      | the encoding of `x`                     |                                          |
| seq, tuple, tuple struct, tuple variant        | `(Short)Array`                          | array                                    |
| struct, struct variant content                 | `(Short)Object`                         | plain object                             |
| map                                            | `(Short)Object`                         | plain object                             |
| unit variant (externally tagged)               | `(Short)StringUtf8 "Variant"`           | `string`                                 |
| newtype/tuple/struct variant (externally tagged) | 1-entry `Object` `{ "Variant": … }`   | object                                   |

Notes:

- **`Vec<u8>` without `serde_bytes`:** serde serializes it as a seq of `u8`, so it becomes an `Array` of `Int32`, exactly as JSON gives `number[]`. Do **not** add special detection for it. Document that `#[serde(with = "serde_bytes")]` gives the compact `ByteArray` form.
- **Bytes and seqs are interchangeable when reading.** The Rust deserializer must accept:
  - a `ByteArray` for a seq-of-`u8` target, using a `SeqAccess` over the bytes;
  - an `Array` of integers in the range 0..=255 for a bytes target.

  This matters because the TS encoder writes a `Uint8Array` as `ByteArray` and a `number[]` as `Array`.
- **Internally tagged, adjacently tagged, untagged and flattened** enums and structs need no special handling. Serde turns them into maps and strings, and those are encoded as above. The deserializer must implement `deserialize_any` correctly, because serde relies on it for these forms.
- **Map keys:** keys must be strings, or integers, `bool` or `char`, which are converted to strings exactly the way `serde_json::ser::MapKeySerializer` does it. Any other key type is an error.
  - When deserializing, a string key can feed an integer, `bool` or `char` target by parsing it. This mirrors `serde_json`'s `MapKey`.
  - Map keys go through the key table like any other object key.
- Fields skipped with `skip_serializing_if` are simply absent from the object.

---

## 3. Rust implementation: new crate `specta-binary`

- Create the crate at `specta-binary/`.
  - Edition 2024, `[lints] workspace = true`.
  - Add it to the `[workspace.dependencies]` internal-crates block.
- Dependencies: `serde`, `thiserror`.
  - It does **not** depend on `specta`, and `specta`/`specta-macros` get no serde-specific changes, as `AGENTS.md` requires.

```
specta-binary/
├── Cargo.toml
├── SPEC.md               # sections 1 + 2 of this task
├── runtime/
│   └── specta_binary.ts  # TS runtime (section 4)
└── src/
    ├── lib.rs            # public API, re-exports, crate docs, TYPESCRIPT_RUNTIME const
    ├── value_type.rs     # ValueType + TryFrom<u8>, MAGIC, VERSION, flag consts
    ├── varint.rs
    ├── keys.rs           # KeyTable: HashMap<Cow<'static, str>, u32> + Vec, first-use ids
    ├── ser.rs            # Serializer
    ├── de.rs             # Deserializer
    └── error.rs          # Error with byte offset + path
```

### 3.1 Public API

```rust
pub fn to_vec<T: ?Sized + Serialize>(value: &T) -> Result<Vec<u8>, Error>;
pub fn to_vec_with<T: ?Sized + Serialize>(value: &T, config: &Config) -> Result<Vec<u8>, Error>;
pub fn to_writer<W: io::Write, T: ?Sized + Serialize>(writer: W, value: &T) -> Result<(), Error>;
pub fn from_slice<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, Error>;
pub fn from_slice_with<'de, T: Deserialize<'de>>(bytes: &'de [u8], config: &Config) -> Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Config { key_table: bool, max_depth: usize }   // builder methods; Default = (true, 128)

pub use value_type::ValueType;
pub struct Serializer { .. }        // impl serde::Serializer for &mut Serializer
pub struct Deserializer<'de> { .. } // impl serde::Deserializer<'de> for &mut Deserializer<'de>

/// The TS runtime source, so users can write it into their frontend from a build script.
pub const TYPESCRIPT_RUNTIME: &str = include_str!("../runtime/specta_binary.ts");
```

- `key_table` is the per-serialize option. `from_slice` reads the header flag and needs no key-table config. Its `Config` only matters for `max_depth`.

### 3.2 Serializer notes

- **Serialization is always buffered.** `to_writer` is `to_vec` followed by `write_all`.
  1. Write the body into a `Vec<u8>`. If `key_table` is on, intern each key into a `KeyTable`.
  2. Build the header, and the table if any key was interned.
  3. Return `header ++ table ++ body`, allocating once with `Vec::with_capacity`.
- **Unknown lengths.** `serialize_seq(None)` / `serialize_map(None)` are common, because `flatten` uses `serialize_map(None)`, so the short-vs-long choice can't always be made up front. To handle this:
  1. Record the offset.
  2. Write the elements.
  3. In `end()`, insert the correct tag and length at the offset with `Vec::splice`.

  When the length is known, write the header immediately.
- Where possible, borrow `&'static str` struct field names into the key table as `Cow::Borrowed` to avoid allocations.
- Return `Error` instead of panicking. The workspace denies `unwrap_used` and `panic`.

### 3.3 Deserializer notes

- Zero-copy where possible:
  - `visit_borrowed_str` / `visit_borrowed_bytes` for values.
  - Keys are `&'de str` slices into the table, or into the inline key bytes.
- `deserialize_any` dispatches on the tag:
  - `Int32` → `visit_i32`
  - `Int64` → `visit_i64`
  - `Float32` → `visit_f32`
  - `(Short)ByteArray` → `visit_borrowed_bytes`
  - `(Short)Object` → `MapAccess` whose keys are `visit_borrowed_str`
  - and so on for the other tags.
- Typed hints follow the leniency rules in 1.5 and the notes in section 2.
- `deserialize_option`: `Null` → `visit_none`. For anything else, do **not** consume the tag; call `visit_some(self)`.
- `deserialize_enum`: a string → unit variant; an object with exactly 1 entry → variant plus content. This is the same logic as `serde_json`.
- Track the depth and a path (field / index stack) so that errors read like `at $.users[3].email: expected string, found Int32 (offset 412)`.

---

## 4. TypeScript runtime (`specta-binary/runtime/specta_binary.ts`)

This is a single, dependency-free, hand-written ES module. It must work in browsers and in Node 18+.

- Nothing generates it.
- `specta-typescript` does not touch it.
- Users copy it in, write it out with `specta_binary::TYPESCRIPT_RUNTIME`, or later get it from an npm package (see Future work).

### 4.1 API

```ts
export const ValueType = { Null: 0, Bool: 5, Int32: 10, /* … */ ShortObject: 65 } as const;

export interface EncodeOptions {
  keyTable?: boolean;               // default true
}

export interface DecodeOptions {
  int64?: "auto" | "number" | "bigint";  // default "auto"
  bytes?: "uint8array" | "array";        // default "uint8array"
  float32?: "shortest" | "exact";        // default "shortest"
  maxDepth?: number;                     // default 128
}

export function encode(value: unknown, options?: EncodeOptions): Uint8Array;
export function decode<T = unknown>(bytes: Uint8Array, options?: DecodeOptions): T;

export class SpectaBinaryError extends Error { readonly path: string; readonly offset: number }
```

Usage with the types `specta-typescript` generates:

```ts
import type { User } from "./bindings";
import { decode, encode } from "./specta_binary";

const user = decode<User>(new Uint8Array(await res.arrayBuffer()));
await fetch("/user", { method: "POST", body: encode(user) });
```

`decode<T>` is an unchecked cast, just like `JSON.parse(...) as T`. That is the cost of having no schema, and it should be documented.

### 4.2 Decoding rules

| Wire                     | JS value                                                                  |
|--------------------------|---------------------------------------------------------------------------|
| `Null`                   | `null`                                                                    |
| `Bool`                   | `boolean`                                                                 |
| `Int32`                  | `number`                                                                  |
| `Int64`                  | depends on `int64`, see below                                             |
| `Float64`                | `number`                                                                  |
| `Float32`                | depends on `float32`, see below                                           |
| `(Short)StringUtf8`      | `string`                                                                  |
| `(Short)ByteArray`       | `Uint8Array` (a `subarray` view, no copy), or `number[]` with `bytes: "array"` |
| `(Short)Array`           | array                                                                     |
| `(Short)Object`          | plain object, properties assigned in wire order                           |

**`int64` modes:**

- `"auto"`: a `number` if `Number.isSafeInteger`, else a `bigint`.
- `"number"`: always a `number`. This may lose precision, the same as `JSON.parse`.
- `"bigint"`: always a `bigint`.

**`float32` modes:**

- `"shortest"`: the shortest decimal that round-trips through `Math.fround`. Find the smallest `p` in 1..=9 where `Math.fround(+x.toPrecision(p)) === x`. This matches `serde_json`'s output for `f32`, so `0.1f32` decodes to `0.1` rather than `0.10000000149011612`.
- `"exact"`: the raw `getFloat32` value. This is faster.

**Object keys:** use `Object.defineProperty`, or assign onto an object created with `Object.create(null)` and then spread, so that a key named `__proto__` can't pollute the prototype. Add a test for this.

**Reader implementation:**

- One `DataView` and one byte offset.
- Short tags read their length with a single `getUint8` call, with no varint loop.
- The key table is decoded once, up front, into a `string[]`.
- Use a shared `TextDecoder` (`fatal: true`).
- Enforce the limits in 1.6.
- Track a path so that errors carry `path` and `offset`.

### 4.3 Encoding rules

The rules mirror `JSON.stringify`, except that unsupported values throw instead of being silently dropped.

| JS value                         | Wire                                                                 |
|----------------------------------|----------------------------------------------------------------------|
| `null`                           | `Null`                                                               |
| `undefined` as an object property | property omitted, like JSON (makes `skip_serializing_if` / `Option` fields work) |
| `undefined` in an array          | `Null`, like JSON                                                    |
| `undefined` at the root          | throw                                                                |
| `boolean`                        | `Bool`                                                               |
| `number`, integer                | `Int32` if in `i32` range; else `Int64` if `Number.isSafeInteger`    |
| `number`, otherwise              | `Float64`. This includes non-safe integers, `NaN` and `±Infinity`    |
| `bigint`                         | `Int32`/`Int64` (narrowest); throw if outside the `i64` range        |
| `string`                         | `(Short)StringUtf8`                                                  |
| `Uint8Array`                     | `(Short)ByteArray`                                                   |
| array                            | `(Short)Array`                                                       |
| object with a `toJSON()` method  | encode the result of `toJSON()`, like JSON (for example `Date`)      |
| plain object                     | `(Short)Object` over `Object.keys`                                   |
| `Map`, `Set`, function, `symbol`, other typed arrays | throw, with the path                             |

- `-0` is written as `Int32 0`, which matches JSON's `"0"`.
- Cycles throw. Track the objects currently on the stack in a `Set`.
- `keyTable: true` uses a two-part writer:
  1. The body buffer interns keys in a `Map<string, number>` as they are first used.
  2. `finish()` prepends the header and the table. If no key was interned, it writes `flags = 0` and no table.
- The output must follow the same short/long rules and first-use key order as Rust, so that TS and Rust produce **byte-identical** output for the same data (see the caveats in 5.4).
- The writer uses a growable `Uint8Array` that doubles in size as needed, plus a `DataView` and a shared `TextEncoder`. Use `encodeInto` with a precomputed upper bound of `str.length * 3`. Write the short form when the actual byte length is ≤ 255, otherwise shift the bytes after writing the varint.

---

## 5. Tests

Follow `AGENTS.md`:

- Put the Rust tests in `specta-binary/tests/` and in `tests/tests/binary.rs`.
- Use `insta` snapshots.
- Keep any temporary project inside the workspace, not in `/tmp`.

1. **Golden bytes:** `insta` snapshots of hex dumps for:
   - the three examples in 1.7;
   - each short/long boundary: lengths 255 and 256 for strings, bytes, arrays and objects;
   - integer boundaries: `i32::MAX`, `i32::MAX as i64 + 1`, `i64::MIN`;
   - `u64::MAX` (an error), `NaN`, `-0.0`;
   - `Vec<u8>` vs `serde_bytes`;
   - an empty seq, map and struct.
2. **Rust round-trips** (`from_slice(to_vec(x)) == x`), with the key table both on and off, for:
   - structs, tuple structs, unit structs;
   - every serde enum representation (external, internal, adjacent, untagged);
   - `flatten` (the unknown-length path), `skip_serializing_if`, `Option<Option<T>>`;
   - `HashMap<u32, T>`, `BTreeMap<String, T>`;
   - recursive types, `serde_json::Value`, `char`, `i128` in and out of range.

   Also check that the same value with and without the key table deserializes to the same thing.
3. **Leniency:**
   - `Int32` → `f32`/`f64` targets;
   - `Float64` → `f32`;
   - `ByteArray` ↔ `Vec<u8>`/`serde_bytes`;
   - a long tag carrying a short length.
4. **Cross-language conformance:** add a small TS test project at `tests/binary-ts/`. Run it with `node --test`, using `--experimental-strip-types` or `tsx` if that is unavailable. Skip it when `node` is not on `PATH`.
   1. Rust writes, for each fixture, `fixtures/<name>.bin` (from `to_vec`, with the table both on and off) and `fixtures/<name>.json` (from `serde_json::to_string`).
   2. The TS test runs `decode(bin)` and `deepStrictEqual`s the result against `JSON.parse(json)`, with default options apart from `bytes: "array"`. Keep `NaN`/`Infinity` out of these fixtures, because JSON can't represent them.
   3. The TS test runs `encode(decoded)` and asserts the bytes are **identical** to the `.bin`. Exclude fixtures where this can't hold:
      - fixtures containing `f32`, because TS re-encodes floats as `Float64`;
      - fixtures containing `serde_bytes` with `bytes: "array"`;
      - fixtures with integer-like string keys that are out of numeric order, because JS reorders them.

      Use `BTreeMap` in fixtures so map order is deterministic.
   4. The TS test `encode`s its own values to `fixtures/ts/<name>.bin`, and a second Rust test `from_slice`s them into the matching Rust types.
5. **Hostile input (Rust and TS):**
   - truncated buffers;
   - bad magic, version or flags;
   - unknown tags;
   - a huge varint length;
   - a key id out of range;
   - depth greater than the maximum;
   - invalid UTF-8;
   - trailing bytes;
   - `__proto__` keys (TS only).

   All must return or throw `SpectaBinaryError` / `Error`. None may panic, hang, or allocate unbounded memory.
6. **Tag parity:** a Rust unit test that parses `runtime/specta_binary.ts`, extracts the `ValueType` object, and asserts it matches the Rust enum.

---

## 6. Docs and example

- Write the `specta-binary` crate docs. Cover the format overview, link `SPEC.md`, show `to_vec`/`from_slice` and the `key_table` option, and recommend `serde_bytes` for byte fields.
- Write a header comment for the TS runtime that covers the API and options, and explains that `decode<T>` is an unchecked cast, like `JSON.parse`.
- Add `examples/binary-ts/`:
  - It derives a few types with `Type + Serialize + Deserialize`.
  - It exports `bindings.ts` with the unchanged `specta-typescript` exporter.
  - It writes `specta_binary.ts` from `TYPESCRIPT_RUNTIME` and a sample `.bin`.
  - Include a short TS snippet that `decode<User>`s the `.bin`.

---

## Acceptance criteria

- [ ] `specta-binary` crate with `SPEC.md`, `to_vec`/`to_vec_with`/`to_writer`/`from_slice`, `Config`, and `TYPESCRIPT_RUNTIME`; no `unwrap`/`panic` in library code.
- [ ] Every tag in `ValueType` is implemented. Short variants are always used for lengths ≤ 255, and readers accept both variants.
- [ ] The key table is optional per serialize call. It is only emitted when the document contains object keys, and decoding needs no configuration.
- [ ] Output is deterministic: the same value gives the same bytes.
- [ ] No schema or code generation anywhere, and `specta-typescript` is not modified.
- [ ] TS `decode` output deep-equals `JSON.parse` of the `serde_json` output for the fixture set.
- [ ] Rust ↔ TS byte-identical round-trips for the eligible fixtures (5.4.3).
- [ ] Malformed input produces errors with path and offset; never a panic, hang, or prototype pollution.
- [ ] `cargo test --workspace --all-features` and `cargo clippy --workspace --all-features` are clean.

## Out of scope / future work

- Publishing the TS runtime as an npm package (`@specta/binary`).
- Interning string *values* (enum variant names, tag values). This would need a new tag, for example `KeyString = 70`, that reuses the key table.
- A true streaming serializer that writes to `io::Write` without buffering. It would require `key_table: false` and known lengths.
- A dedicated `Uint8Array` type in `specta-typescript`, so that `serde_bytes` fields type-check against `decode` output without `bytes: "array"`.
