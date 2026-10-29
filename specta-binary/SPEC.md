# Specta Binary (`SPB`) wire format, version 1

Specta Binary is a compact, self-describing binary encoding of the serde/JSON data model.
It has no schema: like `serde_json` and `JSON.parse`, a reader can decode any document
without knowing its type.

> A decoded value has the same JS shape as `JSON.parse(serde_json::to_string(&value))`.
> The format adds typed numbers, raw bytes, short lengths and interned keys on top of that.

This document is the source of truth for both implementations: the Rust crate
`specta-binary` and the TypeScript runtime in `runtime/specta_binary.ts`.

## 1. Wire format

### 1.1 Conventions

- Fixed-width numbers are **little-endian**. This matches `DataView` with `littleEndian = true`.
- `varint` is an unsigned LEB128 integer, capped at 32 bits (at most 5 bytes).
  Readers must reject varints that are longer than 5 bytes or overflow 32 bits.
  Non-minimal varints (like `81 00` for 1) are accepted.
- `u8len` is a single unsigned byte, 0–255, read directly with **no varint decoding**.
- Strings are UTF-8 and are not NUL-terminated.

### 1.2 Value type tags

Every value is preceded by a one-byte tag:

| Tag               | Byte          | Payload after the tag                                             |
|-------------------|---------------|-------------------------------------------------------------------|
| `Null`            | `0`  (`0x00`) | nothing                                                           |
| `Bool`            | `5`  (`0x05`) | 1 byte: `0x00` = false, `0x01` = true. Any other byte is an error |
| `Int32`           | `10` (`0x0A`) | 4 bytes, `i32` LE                                                 |
| `Int64`           | `15` (`0x0F`) | 8 bytes, `i64` LE                                                 |
| `Float32`         | `20` (`0x14`) | 4 bytes, IEEE-754 `f32` LE                                        |
| `Float64`         | `25` (`0x19`) | 8 bytes, IEEE-754 `f64` LE                                        |
| `StringUtf8`      | `30` (`0x1E`) | `varint byteLen`, then `byteLen` UTF-8 bytes                      |
| `ShortStringUtf8` | `35` (`0x23`) | `u8len byteLen`, then `byteLen` UTF-8 bytes                       |
| `ByteArray`       | `40` (`0x28`) | `varint byteLen`, then `byteLen` raw bytes                        |
| `ShortByteArray`  | `45` (`0x2D`) | `u8len byteLen`, then `byteLen` raw bytes                         |
| `Array`           | `50` (`0x32`) | `varint count`, then `count` tagged values                        |
| `ShortArray`      | `55` (`0x37`) | `u8len count`, then `count` tagged values                         |
| `Object`          | `60` (`0x3C`) | `varint count`, then `count` × (`key`, tagged value). See 1.4     |
| `ShortObject`     | `65` (`0x41`) | `u8len count`, then `count` × (`key`, tagged value). See 1.4      |

**Short-form rules:**

- **Writers** must use the short variant whenever the length or count is ≤ 255, and the long
  variant otherwise. This keeps the output deterministic.
- **Readers** must accept both variants for any length. A long variant holding a length ≤ 255
  is still valid input.

Readers must reject unknown tags with an error that includes the byte offset. The gaps
between tag values are reserved for future tags.

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
  - Ids are assigned in **first-use order** during a depth-first walk of the value: an
    object's key is interned before its value is written. This keeps the output deterministic.
  - The table holds **only object keys**. Strings that are values are never interned.
- After the header (and table, if present) comes **exactly one** root value. Trailing bytes
  are an error.

### 1.4 Object keys

The `HAS_KEY_TABLE` flag decides how every key inside an `Object`/`ShortObject` is encoded,
for the whole document:

| `HAS_KEY_TABLE` | Key encoding                                                           |
|-----------------|------------------------------------------------------------------------|
| set             | `varint keyId`. It must be < `keyCount`                                |
| clear           | `varint byteLen`, then `byteLen` UTF-8 bytes (the key written inline)  |

Keys are never tagged, because they are always strings.

Whether to use a key table is the writer's choice, per document. When the writer has it enabled:

- If the document contains **no object keys**, it emits `flags = 0` and no table.
- An empty table is never written. (Readers accept one.)

### 1.5 Numbers

**Writers:**

- Integers use the **narrowest tag that holds the value losslessly**: `Int32` if it is in
  `i32` range, otherwise `Int64`.
- `u64` values above `i64::MAX` are a serialization error. This avoids a silent sign flip,
  since there is no schema to say the value is unsigned.
- `i128`/`u128` follow the same rule: they are written if they fit in `i64`, otherwise it is an error.
- Rust `f32` → `Float32`, Rust `f64` → `Float64`. Floats are never narrowed.
- JavaScript only has `number`. The TS encoder writes integer values (`Number.isInteger`) as
  `Int32`, or `Int64` if they are safe integers, and everything else as `Float64`.
- `NaN` and `±Infinity` are allowed. This is a deliberate difference from JSON.

**Readers:** with no schema, readers must be lenient. The Rust deserializer:

- accepts `Int32` or `Int64` for any integer target, and range-checks the value;
- accepts **any** numeric tag for a float target, because JS cannot tell `1.0` from `1`;
- converts `f64` to `f32` with `as`, the same way `serde_json` does;
- treats anything else as a type-mismatch error that includes the path and offset.

### 1.6 Limits

Both readers enforce these limits against hostile or corrupted input:

- Maximum nesting depth of arrays and objects: 128 by default, configurable. A document
  may open at most `maxDepth` containers inside each other.
- Each length or count must be ≤ the number of bytes remaining after it, checked before allocating.
- Each key id must be < `keyCount`.
- String bytes (values, inline keys and table keys) must be valid UTF-8.

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

## 2. Mapping the serde data model to wire values

This is the same as `serde_json`. Differences are called out.

| serde / Rust                                       | Wire                                    | `decode` result in TS               |
|----------------------------------------------------|-----------------------------------------|-------------------------------------|
| `bool`                                             | `Bool`                                  | `boolean`                           |
| any integer                                        | `Int32` / `Int64` (narrowest, see 1.5)  | `number`, or `bigint` (see below)   |
| `f32` / `f64`                                      | `Float32` / `Float64`                   | `number`                            |
| `char`, `str`, `String`                            | `(Short)StringUtf8`                     | `string`                            |
| `serialize_bytes` (`serde_bytes`, `bytes::Bytes`)  | `(Short)ByteArray`                      | `Uint8Array` (see below)            |
| `()`, unit struct, `None`                          | `Null`                                  | `null`                              |
| `Some(x)`, newtype struct                          | the encoding of `x`                     |                                     |
| seq, tuple, tuple struct                           | `(Short)Array`                          | array                               |
| struct, map                                        | `(Short)Object`                         | plain object                        |
| unit variant (externally tagged)                   | `(Short)StringUtf8 "Variant"`           | `string`                            |
| newtype/tuple/struct variant (externally tagged)   | 1-entry `Object` `{ "Variant": … }`     | object                              |

Notes:

- **Human-readable.** The Rust serializer and deserializer report `is_human_readable() == true`,
  so types like `Uuid`, `DateTime` and `IpAddr` use the same string forms as in JSON.
- **`Vec<u8>` without `serde_bytes`:** serde serializes it as a seq of `u8`, so it becomes an
  `Array` of `Int32`, exactly as JSON gives `number[]`. Use `#[serde(with = "serde_bytes")]`
  for the compact `ByteArray` form.
- **Bytes and seqs are interchangeable when reading.** The Rust deserializer accepts a
  `ByteArray` for a seq-of-`u8` target and an `Array` of integers in `0..=255` for a bytes
  target, because the TS encoder writes a `Uint8Array` as `ByteArray` and a `number[]` as `Array`.
  (`serde_json::Value` has no variant for raw bytes, so a `ByteArray` cannot be read into it.)
- **Internally tagged, adjacently tagged, untagged and flattened** enums and structs need no
  special handling. Serde turns them into maps and strings, encoded as above. Flattened
  structs serialize as maps of unknown length, so writers must fix up the container header
  once the count is known.
- **Map keys** must be strings, or integers, `bool` or `char`, which are converted to strings
  the way `serde_json` does (`1` → `"1"`, `true` → `"true"`). Unit variants and newtype structs
  use their inner string. Any other key type is an error. When reading, a string key can feed
  an integer, float, `bool` or `char` target by parsing it. Map keys go through the key table
  like any other object key.
- Fields skipped with `skip_serializing_if` are simply absent from the object.

### 2.1 TypeScript decoding

| Wire                | JS value                                                                       |
|---------------------|--------------------------------------------------------------------------------|
| `Null`              | `null`                                                                         |
| `Bool`              | `boolean`                                                                      |
| `Int32`             | `number`                                                                       |
| `Int64`             | `int64: "auto"` (default): `number` if `Number.isSafeInteger`, else `bigint`. `"number"`: always `number`, possibly rounded like `JSON.parse`. `"bigint"`: always `bigint` |
| `Float64`           | `number`                                                                       |
| `Float32`           | `float32: "shortest"` (default): the shortest decimal that rounds to the same `f32`, matching `serde_json` (`0.1f32` → `0.1`). `"exact"`: the raw `getFloat32` value |
| `(Short)StringUtf8` | `string`                                                                       |
| `(Short)ByteArray`  | `bytes: "uint8array"` (default): a `Uint8Array` view into the input (no copy). `"array"`: `number[]` |
| `(Short)Array`      | array                                                                          |
| `(Short)Object`     | plain object, properties assigned in wire order. A `__proto__` key becomes an own property; it never replaces the prototype |

### 2.2 TypeScript encoding

Encoding mirrors `JSON.stringify`, except that unsupported values throw instead of being dropped:

| JS value                                            | Wire                                                         |
|-----------------------------------------------------|--------------------------------------------------------------|
| `null`                                              | `Null`                                                       |
| `undefined` object property                         | omitted, like JSON                                           |
| `undefined` array item                              | `Null`, like JSON                                            |
| `undefined` at the root                             | error                                                        |
| `boolean`                                           | `Bool`                                                       |
| integer `number`                                    | `Int32` if in `i32` range, else `Int64` if a safe integer    |
| other `number` (incl. unsafe integers, `NaN`, `±Infinity`) | `Float64`                                             |
| `-0`                                                | `Int32 0`, like JSON's `0`                                   |
| `bigint`                                            | `Int32`/`Int64` (narrowest); error outside the `i64` range   |
| `string`                                            | `(Short)StringUtf8`                                          |
| `Uint8Array`                                        | `(Short)ByteArray`                                           |
| array                                               | `(Short)Array`                                               |
| object with `toJSON()`                              | the encoding of `toJSON()`'s result, like JSON (e.g. `Date`) |
| other object                                        | `(Short)Object` over `Object.keys`                           |
| `Map`, `Set`, function, `symbol`, other typed arrays, cycles | error, with the path                                |

### 2.3 Byte-identical round trips

For the same data, Rust and TypeScript produce byte-identical documents, with these exceptions,
where `encode(decode(bytes))` differs from `bytes` even though the data is equivalent:

- Floats with integral values (`2.0`, `-0.0`): JS re-encodes them as integers.
- `f32` values: JS re-encodes every float as `Float64`.
- Bytes decoded with `bytes: "array"`: JS re-encodes the `number[]` as an `Array`.
- Unsafe integers decoded with `int64: "number"`: JS re-encodes them as `Float64`.
- Objects whose keys look like integers (`"10"`, `"2"`) and are not in ascending numeric order:
  JS objects iterate those keys first, in numeric order.

### 2.4 Errors

Errors carry the path to the failing value (`$`, `.field`, `["odd key"]`, `[index]`) and the
byte offset where it starts, for example
`at $.users[3].email: expected string, found Int32 (offset 412)`. Header errors have no path.
