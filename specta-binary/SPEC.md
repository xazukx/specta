# Specta Binary (`SPB`) wire format, version 1

Specta Binary is a compact, self-describing binary encoding of the serde/JSON data model.
It has no schema: like `serde_json` and `JSON.parse`, a reader can decode any document
without knowing its type.

> A decoded value has the same JS shape as `JSON.parse(serde_json::to_string(&value))`.
> The format adds typed integers, exact decimals, raw bytes, short lengths and interned keys
> and strings on top of that.

This document is the source of truth for both implementations: the Rust crate
`specta-binary` and the TypeScript runtime in `runtime/specta_binary.ts`.

## 1. Wire format

### 1.1 Conventions

- Fixed-width numbers are **little-endian**. This matches `DataView` with `littleEndian = true`.
- `varint` is an unsigned LEB128 integer, capped at 32 bits (at most 5 bytes).
  Readers must reject varints that are longer than 5 bytes or overflow 32 bits.
  Non-minimal varints (like `81 00` for 1) are accepted.
- `varint64` is the same encoding capped at 64 bits (at most 10 bytes, the 10th holding only bit 63).
- `zigzag` maps a signed integer to an unsigned one so small magnitudes stay small:
  0, -1, 1, -2, … → 0, 1, 2, 3, ….
- `u8len` is a single unsigned byte, 0–255, read directly with **no varint decoding**.
- Strings are UTF-8 and are not NUL-terminated.

### 1.2 Value type tags

Every value is preceded by a one-byte tag:

| Tag               | Byte          | Payload after the tag                                             |
|-------------------|---------------|-------------------------------------------------------------------|
| `Null`            | `0`  (`0x00`) | nothing                                                           |
| `Bool`            | `5`  (`0x05`) | 1 byte: `0x00` = false, `0x01` = true. Any other byte is an error |
| `Int16`           | `10` (`0x0A`) | 2 bytes, `i16` LE                                                 |
| `Int32`           | `15` (`0x0F`) | 4 bytes, `i32` LE                                                 |
| `Int64`           | `20` (`0x14`) | 8 bytes, `i64` LE                                                 |
| `Decimal`         | `25` (`0x19`) | `varint head`, then `varint64 mantissa`. See 1.6                  |
| `Float64`         | `30` (`0x1E`) | 8 bytes, IEEE-754 `f64` LE                                        |
| `StringUtf8`      | `35` (`0x23`) | `varint byteLen`, then `byteLen` UTF-8 bytes                      |
| `ShortStringUtf8` | `40` (`0x28`) | `u8len byteLen`, then `byteLen` UTF-8 bytes                       |
| `InternedString`  | `45` (`0x2D`) | `varint valueId`, an index into the value table. See 1.5          |
| `ByteArray`       | `50` (`0x32`) | `varint byteLen`, then `byteLen` raw bytes                        |
| `ShortByteArray`  | `55` (`0x37`) | `u8len byteLen`, then `byteLen` raw bytes                         |
| `Array`           | `60` (`0x3C`) | `varint count`, then `count` tagged values                        |
| `ShortArray`      | `65` (`0x41`) | `u8len count`, then `count` tagged values                         |
| `Object`          | `70` (`0x46`) | `varint count`, then `count` × (`key`, tagged value). See 1.4     |
| `ShortObject`     | `75` (`0x4B`) | `u8len count`, then `count` × (`key`, tagged value). See 1.4      |

**Short-form rules:**

- **Writers** must use the short variant whenever the length or count is ≤ 255, and the long
  variant otherwise. This keeps the output deterministic.
- **Readers** must accept both variants for any length. A long variant holding a length ≤ 255
  is still valid input.

Readers must reject unknown tags with an error that includes the byte offset. The gaps
between tag values are reserved for future tags.

### 1.3 Document layout

```
+----------+----------+---------+-------+----------------------+------------------------+-------------------+
| 'S' 0x53 | 'B' 0x42 | version | flags | [key table, if flag] | [value table, if flag] | root tagged value |
+----------+----------+---------+-------+----------------------+------------------------+-------------------+
```

- `version` is `0x01`. Readers must reject versions they do not know.
- `flags` is a bitset:
  - bit 0, `HAS_KEY_TABLE`: a key table follows the header.
  - bit 1, `HAS_VALUE_TABLE`: a value table follows the header, after the key table if there is one.
  - All other bits must be 0, and readers must reject a document if they are set.
- Both tables have the same layout:
  ```
  varint count
  count × ( varint byteLen, byteLen UTF-8 bytes )
  ```
  - A string's id is its zero-based position in its table. Ids are implicit, so they cost no bytes.
  - Ids are assigned in **first-use order** during a depth-first walk of the value, which
    keeps the output deterministic. An object's key is interned before its value is written.
  - The **key table** holds only object keys and the **value table** holds only string
    values. The tables are independent: a string used both as a key and as a value is in both.
- After the header (and tables, if present) comes **exactly one** root value. Trailing bytes
  are an error.

Whether to use each table is the writer's choice, per document. A writer with a table enabled
leaves it out (and clears its flag) when it would be empty. Readers accept empty tables.

### 1.4 Object keys

The `HAS_KEY_TABLE` flag decides how every key inside an `Object`/`ShortObject` is encoded,
for the whole document:

| `HAS_KEY_TABLE` | Key encoding                                                           |
|-----------------|------------------------------------------------------------------------|
| set             | `varint keyId`. It must be < `count` of the key table                  |
| clear           | `varint byteLen`, then `byteLen` UTF-8 bytes (the key written inline)  |

Keys are never tagged, because they are always strings.

### 1.5 String values

A string value is written in one of two ways:

- inline, as `(Short)StringUtf8`;
- interned, as `InternedString` with an id into the value table. The id must be < `count` of
  the value table, and a document without a value table has no valid ids.

A writer with the value table enabled writes **every** string value as `InternedString`,
including strings used only once, so the output stays deterministic. A repeated string then
costs 2–4 bytes after its first use. With the value table disabled, every string is inline.

Readers accept both forms in any document, so inline strings may appear next to interned ones.

### 1.6 Numbers

**Integers** use the **narrowest tag that holds the value losslessly**: `Int16` if it is in
`i16` range, else `Int32` if it is in `i32` range, else `Int64`.

- `u64` values above `i64::MAX` are a serialization error. This avoids a silent sign flip,
  since there is no schema to say the value is unsigned.
- `i128`/`u128` follow the same rule: they are written if they fit in `i64`, otherwise it is an error.
- JavaScript only has `number`. The TS encoder writes safe integers (`Number.isSafeInteger`)
  with an integer tag and every other number as a float, below.

**Floats** are written in one of two forms, both of which read back as exactly the same value:

- `Decimal`: the value as `±mantissa × 10^exponent`. `head` is `zigzag(exponent) × 2 + sign`,
  where `sign` is 1 for negative values (including `-0`), and `mantissa` is a `varint64`.
  `1.5` is `19 02 0F` (15 × 10^-1), `19.99` is `19 06 CF 0F` and `1e21` is `19 54 01`.
- `Float64`: the IEEE-754 bits.

Writers use the shortest mantissa that reads back as the same value of the source type (`f32`
for a Rust `f32`, otherwise `f64`), with no trailing zeros. If two mantissas are equally close
to the value, the even one wins. They then write the `Decimal` if it takes at most 9 bytes
(the size of a `Float64`), and the `Float64` otherwise. `NaN` and `±Infinity` are always
`Float64`. So a float never takes more than 9 bytes, and "human" decimals like prices and
coordinates take 3–6.

In practice up to 15 significant digits fit in a `Decimal` (`0.123456789012345` takes exactly
9 bytes). Two shortest mantissas can only be equally close with 16 or more digits, so every
`f64` that is written as a `Decimal` has a unique shortest mantissa: `{:e}` in Rust and
`String(value)` in JavaScript agree on it. An `f32` can tie with 7 or more digits; Rust detects
that exactly and picks the even mantissa, like `serde_json`.

Readers accept any `head` and `mantissa`, including trailing zeros, and round
`mantissa × 10^exponent` once, to the nearest value of the target type. Extreme exponents
round to `±Infinity` or `±0`.

**Readers:** with no schema, readers must be lenient. The Rust deserializer:

- accepts `Int16`, `Int32` or `Int64` for any integer target, and range-checks the value;
- accepts `Decimal`, `Float64` or any integer tag for a float target, because JS cannot tell
  `1.0` from `1`;
- rounds a `Decimal` straight to the target type, so an `f32` is rounded once, and converts a
  `Float64` to `f32` with `as`, like `serde_json`;
- treats anything else (including a float for an integer target) as a type-mismatch error
  that includes the path and offset.

### 1.7 Limits

Both readers enforce these limits against hostile or corrupted input:

- Maximum nesting depth of arrays and objects: 128 by default, configurable. A document
  may open at most `maxDepth` containers inside each other.
- Each length or count must be ≤ the number of bytes remaining after it, checked before allocating.
- Each key id must be < the key table's `count`, and each value id < the value table's `count`.
- String bytes (values, inline keys and both tables) must be valid UTF-8.

### 1.8 Worked examples

```rust
#[derive(Serialize, Deserialize, Type)]
struct User { name: String, email: Option<String>, age: u8 }

User { name: "Ann".into(), email: None, age: 30 }
```

**With both tables** (the default):

```
53 42 01 03                 header: "SB", v1, HAS_KEY_TABLE | HAS_VALUE_TABLE
03                          key table: 3 keys
04 6E 61 6D 65              [0] "name"
05 65 6D 61 69 6C           [1] "email"
03 61 67 65                 [2] "age"
01                          value table: 1 string
03 41 6E 6E                 [0] "Ann"
4B 03                       ShortObject, 3 entries
00   2D 00                  key#0 -> InternedString #0 ("Ann")
01   00                     key#1 -> Null
02   0A 1E 00               key#2 -> Int16 30
```

**Without tables:**

```
53 42 01 00                 header: "SB", v1, no flags
4B 03                       ShortObject, 3 entries
04 6E 61 6D 65   28 03 41 6E 6E      "name"  -> ShortStringUtf8 "Ann"
05 65 6D 61 69 6C   00               "email" -> Null
03 61 67 65   0A 1E 00               "age"   -> Int16 30
```

**A document with no keys or strings**, for example `vec![1, 2]`, has no tables even when
they are enabled:

```
53 42 01 00   41 02   0A 01 00   0A 02 00
```

**Repeated strings**, for example `["a", "b", "a"]`:

```
53 42 01 02   02 01 61 01 62   41 03   2D 00   2D 01   2D 00
```

**Floats**, for example `(1.5, 19.99, -1e-7, 0.1f32, 0.1 + 0.2, f64::NAN)`:

```
53 42 01 00   41 06
19 02 0F                    Decimal 15 × 10^-1
19 06 CF 0F                 Decimal 1999 × 10^-2
19 1B 01                    Decimal -(1 × 10^-7)
19 02 01                    Decimal 1 × 10^-1 (the shortest decimal for the f32)
1E 34 33 33 33 33 33 D3 3F  Float64 0.30000000000000004 (17 digits don't fit in 9 bytes)
1E 00 00 00 00 00 00 F8 7F  Float64 NaN
```

## 2. Mapping the serde data model to wire values

This is the same as `serde_json`. Differences are called out.

| serde / Rust                                       | Wire                                                   | `decode` result in TS               |
|----------------------------------------------------|--------------------------------------------------------|-------------------------------------|
| `bool`                                             | `Bool`                                                 | `boolean`                           |
| any integer                                        | `Int16` / `Int32` / `Int64` (narrowest, see 1.6)       | `number`, or `bigint` (see below)   |
| `f32` / `f64`                                      | `Decimal`, or `Float64` when that is shorter (see 1.6) | `number`                            |
| `char`, `str`, `String`                            | `InternedString`, or `(Short)StringUtf8` without the value table | `string`                  |
| `serialize_bytes` (`serde_bytes`, `bytes::Bytes`)  | `(Short)ByteArray`                                     | `Uint8Array` (see below)            |
| `()`, unit struct, `None`                          | `Null`                                                 | `null`                              |
| `Some(x)`, newtype struct                          | the encoding of `x`                                    |                                     |
| seq, tuple, tuple struct                           | `(Short)Array`                                         | array                               |
| struct, map                                        | `(Short)Object`                                        | plain object                        |
| unit variant (externally tagged)                   | the string `"Variant"`                                 | `string`                            |
| newtype/tuple/struct variant (externally tagged)   | 1-entry `Object` `{ "Variant": … }`                    | object                              |

Notes:

- **Human-readable.** The Rust serializer and deserializer report `is_human_readable() == true`,
  so types like `Uuid`, `DateTime` and `IpAddr` use the same string forms as in JSON.
- **`Vec<u8>` without `serde_bytes`:** serde serializes it as a seq of `u8`, so it becomes an
  `Array` of `Int16`, exactly as JSON gives `number[]`. Use `#[serde(with = "serde_bytes")]`
  for the compact `ByteArray` form.
- **Bytes and seqs are interchangeable when reading.** The Rust deserializer accepts a
  `ByteArray` for a seq-of-`u8` target and an `Array` of integers in `0..=255` for a bytes
  target, because the TS encoder writes a `Uint8Array` as `ByteArray` and a `number[]` as `Array`.
  (`serde_json::Value` has no variant for raw bytes, so a `ByteArray` cannot be read into it.)
- **Floats** keep their decimal meaning: `0.1f32` is written as 1 × 10^-1, so it reads back as
  `0.1` into an `f64` or in JavaScript (not `0.10000000149011612`), and as the same `f32` in Rust.
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

| Wire                                   | JS value                                                                       |
|----------------------------------------|--------------------------------------------------------------------------------|
| `Null`                                 | `null`                                                                         |
| `Bool`                                 | `boolean`                                                                      |
| `Int16`, `Int32`                       | `number`                                                                       |
| `Int64`                                | `int64: "auto"` (default): `number` if `Number.isSafeInteger`, else `bigint`. `"number"`: always `number`, possibly rounded like `JSON.parse`. `"bigint"`: always `bigint` |
| `Decimal`                              | `number`: `mantissa × 10^exponent` rounded once, the same as `JSON.parse` gives for that decimal |
| `Float64`                              | `number`                                                                       |
| `(Short)StringUtf8`, `InternedString`  | `string`                                                                       |
| `(Short)ByteArray`                     | `bytes: "uint8array"` (default): a `Uint8Array` view into the input (no copy). `"array"`: `number[]` |
| `(Short)Array`                         | array                                                                          |
| `(Short)Object`                        | plain object, properties assigned in wire order. A `__proto__` key becomes an own property; it never replaces the prototype |

### 2.2 TypeScript encoding

Encoding mirrors `JSON.stringify`, except that unsupported values throw instead of being dropped:

| JS value                                            | Wire                                                         |
|-----------------------------------------------------|--------------------------------------------------------------|
| `null`                                              | `Null`                                                       |
| `undefined` object property                         | omitted, like JSON                                           |
| `undefined` array item                              | `Null`, like JSON                                            |
| `undefined` at the root                             | error                                                        |
| `boolean`                                           | `Bool`                                                       |
| safe integer `number`                               | `Int16` / `Int32` / `Int64` (narrowest)                      |
| other `number` (incl. unsafe integers)              | `Decimal` if it takes at most 9 bytes, else `Float64`        |
| `NaN`, `±Infinity`                                  | `Float64`                                                    |
| `-0`                                                | `Int16 0`, like JSON's `0`                                   |
| `bigint`                                            | `Int16`/`Int32`/`Int64` (narrowest); error outside the `i64` range |
| `string`                                            | `InternedString`, or `(Short)StringUtf8` with `valueTable: false` |
| `Uint8Array`, including Node's `Buffer`             | `(Short)ByteArray`, without calling `Buffer`'s `toJSON()`    |
| array                                               | `(Short)Array`                                               |
| object with `toJSON()`                              | the encoding of `toJSON()`'s result, like JSON (e.g. `Date`) |
| other object                                        | `(Short)Object` over `Object.keys`                           |
| `Map`, `Set`, function, `symbol`, other typed arrays, cycles | error, with the path                                |

The mantissa and exponent come from `String(value)`, which gives the shortest digits like Rust,
so both encoders write the same bytes.

### 2.3 Byte-identical round trips

For the same data and the same table options, Rust and TypeScript produce byte-identical
documents, with these exceptions, where `encode(decode(bytes))` differs from `bytes` even though
the data is equivalent:

- Floats with integral values in the safe integer range (`2.0`, `-0.0`): JS re-encodes them as integers.
- Bytes decoded with `bytes: "array"`: JS re-encodes the `number[]` as an `Array`.
- Unsafe integers decoded with `int64: "number"`: JS re-encodes them as a `Decimal` or `Float64`.
- Objects whose keys look like integers (`"10"`, `"2"`) and are not in ascending numeric order:
  JS objects iterate those keys first, in numeric order.

### 2.4 Errors

Errors carry the path to the failing value (`$`, `.field`, `["odd key"]`, `[index]`) and the
byte offset where it starts, for example
`at $.users[3].email: expected string, found Int32 (offset 412)`. Header and table errors have
no path.
