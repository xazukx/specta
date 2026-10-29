/**
 * Specta Binary (SPB) runtime for TypeScript and JavaScript.
 *
 * A dependency-free encoder and decoder for the Specta Binary wire format, the binary
 * counterpart of `serde_json` + `JSON.parse`. There is no schema and no code generation:
 * `decode` returns the same JS shape as `JSON.parse(serde_json::to_string(&value))`.
 * See `SPEC.md` in the `specta-binary` crate for the wire format.
 *
 * Works in browsers and Node 18+ (it needs `BigInt`, `TextEncoder` and `TextDecoder`).
 *
 * ```ts
 * import type { User } from "./bindings";
 * import { decode, encode } from "./specta_binary";
 *
 * const user = decode<User>(new Uint8Array(await res.arrayBuffer()));
 * await fetch("/user", { method: "POST", body: encode(user) });
 * ```
 *
 * `decode<T>` is an unchecked cast, exactly like `JSON.parse(text) as T`: the bytes are
 * not validated against `T`. Validate untrusted input separately if you need to.
 *
 * Decoding (`DecodeOptions`):
 * - `int64`: `"auto"` (default) gives a `number` when it is a safe integer and a `bigint`
 *   otherwise. `"number"` always gives a `number`, possibly losing precision like
 *   `JSON.parse`. `"bigint"` always gives a `bigint`.
 * - `bytes`: `"uint8array"` (default) gives a `Uint8Array` view into the input (no copy, so
 *   it changes if the input does). `"array"` gives a `number[]`, the JSON shape.
 * - `maxDepth`: the maximum nesting of arrays and objects. Defaults to 128.
 *
 * Rust floats arrive as `Decimal` (the shortest decimal, like `1.5` = 15 × 10^-1) or, when that
 * would be longer, as `Float64`. Both decode to the same `number` as `JSON.parse` of the
 * `serde_json` output, so `0.1f32` decodes to `0.1`.
 *
 * Encoding (`EncodeOptions`) mirrors `JSON.stringify`, except that unsupported values throw:
 * - `keyTable`: intern object keys into a table at the start of the document (default `true`).
 * - `valueTable`: intern string values into a table at the start of the document (default `true`).
 * - `undefined` object properties are omitted, `undefined` array items become `null`.
 * - Safe integers are written as `Int16`/`Int32`/`Int64`. Other numbers are written as
 *   `Decimal` when it takes at most 9 bytes, otherwise (and for `NaN` and `±Infinity`) as
 *   `Float64`. `bigint` is written as an integer and must fit in an `i64`.
 * - `Uint8Array` (including Node's `Buffer`) is written as a byte array. Other objects with a
 *   `toJSON()` method (like `Date`) are encoded as its result.
 * - `Map`, `Set`, functions, symbols, other typed arrays and cycles throw.
 *
 * Errors are thrown as `SpectaBinaryError`, which carries the `path` to the failing value
 * (like `$.users[3].email`) and the byte `offset` where it starts (`-1` when encoding).
 */

export const ValueType = {
  Null: 0,
  Bool: 5,
  Int16: 10,
  Int32: 15,
  Int64: 20,
  Decimal: 25,
  Float64: 30,
  StringUtf8: 35,
  ShortStringUtf8: 40,
  InternedString: 45,
  ByteArray: 50,
  ShortByteArray: 55,
  Array: 60,
  ShortArray: 65,
  Object: 70,
  ShortObject: 75,
} as const;

export type ValueType = (typeof ValueType)[keyof typeof ValueType];

export interface EncodeOptions {
  /** Intern object keys into a key table. Defaults to `true`. */
  keyTable?: boolean;
  /** Intern string values into a value table. Defaults to `true`. */
  valueTable?: boolean;
}

export interface DecodeOptions {
  /** How to return `Int64` values. Defaults to `"auto"`. */
  int64?: "auto" | "number" | "bigint";
  /** How to return byte arrays. Defaults to `"uint8array"`. */
  bytes?: "uint8array" | "array";
  /** The maximum nesting depth of arrays and objects. Defaults to `128`. */
  maxDepth?: number;
}

const VERSION = 0x01;
const FLAG_HAS_KEY_TABLE = 0x01;
const FLAG_HAS_VALUE_TABLE = 0x02;
const SHORT_MAX = 0xff;
const TWO_POW_32 = 0x1_0000_0000;
const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;

const PREPEND = Symbol("prepend");
const ROOT = Symbol("root");

/** Thrown for malformed input when decoding, and for unsupported values when encoding. */
export class SpectaBinaryError extends Error {
  /** What went wrong, without the path and offset. */
  readonly reason: string;
  /** The byte offset where the failing value (or header field) starts, or `-1` when encoding. */
  readonly offset: number;
  #path: string | null = null;

  constructor(reason: string, offset = -1) {
    super(reason);
    this.name = "SpectaBinaryError";
    this.reason = reason;
    this.offset = offset;
    this.#format();
  }

  /** The path to the failing value, like `$.users[3].email`. Empty for header errors. */
  get path(): string {
    return this.#path === null ? "" : `$${this.#path}`;
  }

  [PREPEND](segment: string): this {
    this.#path = segment + (this.#path ?? "");
    this.#format();
    return this;
  }

  [ROOT](): this {
    this.#path ??= "";
    this.#format();
    return this;
  }

  #format(): void {
    const at = this.#path === null ? "" : `at ${this.path}: `;
    const offset = this.offset >= 0 ? ` (offset ${this.offset})` : "";
    this.message = `${at}${this.reason}${offset}`;
  }
}

function prepend(err: unknown, segment: string): unknown {
  return err instanceof SpectaBinaryError ? err[PREPEND](segment) : err;
}

function rooted(err: unknown): unknown {
  return err instanceof SpectaBinaryError ? err[ROOT]() : err;
}

function keySegment(key: string): string {
  return /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(key) ? `.${key}` : `[${JSON.stringify(key)}]`;
}

/** The size of a `Float64` value. Writers use `Decimal` when it is no larger. */
const FLOAT64_LEN = 9;
/** The powers of ten that are exact in a `number`. */
const POW10 = Array.from({ length: 23 }, (_, i) => Number(`1e${i}`));

function hex(byte: number): string {
  return `0x${byte.toString(16).padStart(2, "0")}`;
}

// `ignoreBOM` keeps a leading U+FEFF in the string instead of stripping it.
const textDecoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const textEncoder = new TextEncoder();

// ---------------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------------

/**
 * Decode a Specta Binary document. `T` is an unchecked cast, like `JSON.parse(text) as T`.
 */
export function decode<T = unknown>(bytes: Uint8Array, options?: DecodeOptions): T {
  const reader = new Reader(bytes, options ?? {});
  reader.header();
  let value: unknown;
  try {
    value = reader.value();
  } catch (err) {
    throw rooted(err);
  }
  if (reader.pos !== reader.end) {
    throw new SpectaBinaryError("trailing bytes after the root value", reader.pos);
  }
  return value as T;
}

class Reader {
  readonly bytes: Uint8Array;
  readonly view: DataView;
  readonly end: number;
  pos = 0;
  keys: string[] | null = null;
  values: string[] = [];
  depth = 0;
  readonly int64: "auto" | "number" | "bigint";
  readonly bytesAsArray: boolean;
  readonly maxDepth: number;

  constructor(bytes: Uint8Array, options: DecodeOptions) {
    this.bytes = bytes;
    this.view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    this.end = bytes.byteLength;
    this.int64 = options.int64 ?? "auto";
    this.bytesAsArray = options.bytes === "array";
    this.maxDepth = options.maxDepth ?? 128;
  }

  header(): void {
    this.need(2);
    if (this.view.getUint8(0) !== 0x53 || this.view.getUint8(1) !== 0x42) {
      throw new SpectaBinaryError('invalid magic bytes, expected "SB"', 0);
    }
    this.pos = 2;
    const version = this.u8();
    if (version !== VERSION) {
      throw new SpectaBinaryError(`unsupported version ${version}`, 2);
    }
    const flags = this.u8();
    if ((flags & ~(FLAG_HAS_KEY_TABLE | FLAG_HAS_VALUE_TABLE)) !== 0) {
      throw new SpectaBinaryError(`unknown header flags ${hex(flags)}`, 3);
    }
    if (flags & FLAG_HAS_KEY_TABLE) {
      this.keys = this.table();
    }
    if (flags & FLAG_HAS_VALUE_TABLE) {
      this.values = this.table();
    }
  }

  table(): string[] {
    const count = this.varLen();
    const strings: string[] = [];
    for (let i = 0; i < count; i++) {
      strings.push(this.str(this.varLen()));
    }
    return strings;
  }

  need(n: number): void {
    if (this.end - this.pos < n) {
      throw new SpectaBinaryError("unexpected end of input", this.pos);
    }
  }

  u8(): number {
    this.need(1);
    return this.view.getUint8(this.pos++);
  }

  varint(): number {
    const at = this.pos;
    let value = 0;
    for (let i = 0; i < 5; i++) {
      if (this.pos >= this.end) {
        throw new SpectaBinaryError("unexpected end of input", at);
      }
      const byte = this.view.getUint8(this.pos++);
      // The 5th byte holds bits 28..32, so only its low nibble may be set and it must be the last.
      if (i === 4 && byte > 0x0f) {
        break;
      }
      value += (byte & 0x7f) * 2 ** (7 * i);
      if ((byte & 0x80) === 0) {
        return value;
      }
    }
    throw new SpectaBinaryError("varint is too long or overflows", at);
  }

  /** A 64-bit varint, as a `number` while that is exact and a `bigint` beyond 7 bytes. */
  varint64(): number | bigint {
    const at = this.pos;
    let value = 0;
    let big: bigint | null = null;
    for (let i = 0; i < 10; i++) {
      if (this.pos >= this.end) {
        throw new SpectaBinaryError("unexpected end of input", at);
      }
      const byte = this.view.getUint8(this.pos++);
      // The 10th byte holds bit 63, so only its lowest bit may be set and it must be the last.
      if (i === 9 && byte > 0x01) {
        break;
      }
      if (i < 7) {
        value += (byte & 0x7f) * 2 ** (7 * i);
      } else {
        big = (big ?? BigInt(value)) + (BigInt(byte & 0x7f) << BigInt(7 * i));
      }
      if ((byte & 0x80) === 0) {
        return big ?? value;
      }
    }
    throw new SpectaBinaryError("varint is too long or overflows", at);
  }

  /** Check a length or count read at `at` against the bytes that are left. */
  len(at: number, len: number): number {
    const remaining = this.end - this.pos;
    if (len > remaining) {
      throw new SpectaBinaryError(`length ${len} exceeds the ${remaining} remaining bytes`, at);
    }
    return len;
  }

  varLen(): number {
    const at = this.pos;
    return this.len(at, this.varint());
  }

  shortLen(): number {
    const at = this.pos;
    return this.len(at, this.u8());
  }

  str(len: number): string {
    const start = this.pos;
    this.pos += len;
    // Short ASCII strings are cheaper to build by hand than through `TextDecoder`.
    if (len <= 16) {
      let out = "";
      for (let i = start; i < start + len; i++) {
        const byte = this.view.getUint8(i);
        if (byte > 0x7f) {
          return this.utf8(start, len);
        }
        out += String.fromCharCode(byte);
      }
      return out;
    }
    return this.utf8(start, len);
  }

  utf8(start: number, len: number): string {
    try {
      return textDecoder.decode(this.bytes.subarray(start, start + len));
    } catch {
      throw new SpectaBinaryError("invalid UTF-8", start);
    }
  }

  key(): string {
    const at = this.pos;
    const n = this.varint();
    if (this.keys === null) {
      return this.str(this.len(at, n));
    }
    const key = this.keys[n];
    if (key === undefined) {
      throw new SpectaBinaryError(
        `key id ${n} is out of range for a key table of ${this.keys.length} keys`,
        at,
      );
    }
    return key;
  }

  interned(): string {
    const at = this.pos;
    const id = this.varint();
    const value = this.values[id];
    if (value === undefined) {
      throw new SpectaBinaryError(
        `value id ${id} is out of range for a value table of ${this.values.length} values`,
        at,
      );
    }
    return value;
  }

  /** `±mantissa × 10^exponent`, rounded to the nearest `number`. */
  decimal(): number {
    const head = this.varint();
    const mantissa = this.varint64();
    const zigzag = head >>> 1;
    const exponent = (zigzag >>> 1) ^ -(zigzag & 1);
    let value: number;
    if (typeof mantissa === "number" && mantissa <= 2 ** 53 && Math.abs(exponent) <= 22) {
      // Both operands are exact, so a single operation rounds correctly.
      value = exponent < 0 ? mantissa / POW10[-exponent]! : mantissa * POW10[exponent]!;
    } else {
      value = Number(`${mantissa}e${exponent}`);
    }
    return head & 1 ? -value : value;
  }

  enter(at: number): void {
    if (this.depth >= this.maxDepth) {
      throw new SpectaBinaryError(`maximum nesting depth of ${this.maxDepth} exceeded`, at);
    }
    this.depth++;
  }

  value(): unknown {
    const at = this.pos;
    const tag = this.u8();
    switch (tag) {
      case ValueType.Null:
        return null;
      case ValueType.Bool: {
        const byte = this.u8();
        if (byte > 1) {
          throw new SpectaBinaryError(`invalid bool byte ${hex(byte)}`, this.pos - 1);
        }
        return byte === 1;
      }
      case ValueType.Int16: {
        this.need(2);
        const value = this.view.getInt16(this.pos, true);
        this.pos += 2;
        return value;
      }
      case ValueType.Int32: {
        this.need(4);
        const value = this.view.getInt32(this.pos, true);
        this.pos += 4;
        return value;
      }
      case ValueType.Int64:
        return this.int64Value();
      case ValueType.Decimal:
        return this.decimal();
      case ValueType.Float64: {
        this.need(8);
        const value = this.view.getFloat64(this.pos, true);
        this.pos += 8;
        return value;
      }
      case ValueType.StringUtf8:
        return this.str(this.varLen());
      case ValueType.ShortStringUtf8:
        return this.str(this.shortLen());
      case ValueType.InternedString:
        return this.interned();
      case ValueType.ByteArray:
        return this.byteArray(this.varLen());
      case ValueType.ShortByteArray:
        return this.byteArray(this.shortLen());
      case ValueType.Array:
        return this.array(this.varLen(), at);
      case ValueType.ShortArray:
        return this.array(this.shortLen(), at);
      case ValueType.Object:
        return this.object(this.varLen(), at);
      case ValueType.ShortObject:
        return this.object(this.shortLen(), at);
      default:
        throw new SpectaBinaryError(`unknown value type tag ${hex(tag)}`, at);
    }
  }

  int64Value(): number | bigint {
    this.need(8);
    const at = this.pos;
    this.pos += 8;
    if (this.int64 === "bigint") {
      return this.view.getBigInt64(at, true);
    }
    // `hi * 2^32 + lo` is exact for safe integers and correctly rounded otherwise.
    const lo = this.view.getUint32(at, true);
    const hi = this.view.getInt32(at + 4, true);
    const value = hi * TWO_POW_32 + lo;
    if (this.int64 === "number" || Number.isSafeInteger(value)) {
      return value;
    }
    return this.view.getBigInt64(at, true);
  }

  byteArray(len: number): Uint8Array | number[] {
    const bytes = this.bytes.subarray(this.pos, this.pos + len);
    this.pos += len;
    return this.bytesAsArray ? Array.from(bytes) : bytes;
  }

  array(count: number, at: number): unknown[] {
    this.enter(at);
    const out: unknown[] = [];
    let i = 0;
    try {
      for (; i < count; i++) {
        out.push(this.value());
      }
    } catch (err) {
      throw prepend(err, `[${i}]`);
    }
    this.depth--;
    return out;
  }

  object(count: number, at: number): Record<string, unknown> {
    this.enter(at);
    const out: Record<string, unknown> = {};
    let key: string | undefined;
    try {
      for (let i = 0; i < count; i++) {
        key = undefined;
        key = this.key();
        const value = this.value();
        if (key === "__proto__") {
          // Assigning would call the `__proto__` setter and replace the prototype.
          Object.defineProperty(out, key, {
            value,
            enumerable: true,
            writable: true,
            configurable: true,
          });
        } else {
          out[key] = value;
        }
      }
    } catch (err) {
      throw key === undefined ? err : prepend(err, keySegment(key));
    }
    this.depth--;
    return out;
  }
}

// ---------------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------------

/** Encode a value as a Specta Binary document. */
export function encode(value: unknown, options?: EncodeOptions): Uint8Array {
  const writer = new Writer(options?.keyTable ?? true, options?.valueTable ?? true);
  try {
    const root = resolve(value, "");
    if (root === undefined) {
      throw new SpectaBinaryError("cannot encode undefined");
    }
    writer.value(root);
  } catch (err) {
    throw rooted(err);
  }
  return writer.finish();
}

/**
 * Apply `toJSON`, like `JSON.stringify` does. Byte arrays skip it, because Node's
 * `Buffer.toJSON()` would turn the bytes into `{ type: "Buffer", data: number[] }`.
 */
function resolve(value: unknown, key: string | number): unknown {
  if (typeof value === "object" && value !== null && !(value instanceof Uint8Array)) {
    const toJSON = (value as { toJSON?: unknown }).toJSON;
    if (typeof toJSON === "function") {
      return toJSON.call(value, String(key));
    }
  }
  return value;
}

/**
 * A finite, non-integer or unsafe `value` as `±mantissa × 10^exponent`, with the `head` varint
 * holding the zigzag-encoded exponent and the sign. `String` gives the shortest digits that read
 * back as `value`, like `1.5`, `0.001`, `1e+21` or `123456789012345680000`. A mantissa of 16 or
 * more digits isn't exact here, but it is too long for a `Decimal` anyway.
 */
function shortestDecimal(value: number): { head: number; mantissa: number } {
  const [digits = "", exp = "0"] = String(Math.abs(value)).split("e");
  const [int = "", fraction = ""] = digits.split(".");
  const all = int + fraction;
  const significant = all.replace(/0+$/, "");
  const exponent = Number(exp) - fraction.length + (all.length - significant.length);
  const zigzag = exponent < 0 ? -exponent * 2 - 1 : exponent * 2;
  return { head: zigzag * 2 + (value < 0 ? 1 : 0), mantissa: Number(significant) };
}

function varintLen(value: number): number {
  let len = 1;
  while (value >= 0x80) {
    value = Math.floor(value / 0x80);
    len++;
  }
  return len;
}

class Writer {
  buf = new Uint8Array(256);
  view = new DataView(this.buf.buffer);
  pos = 0;
  readonly keys: Map<string, number> | null;
  readonly values: Map<string, number> | null;
  /** The arrays and objects currently being encoded, to detect cycles. */
  readonly stack = new Set<object>();

  constructor(keyTable: boolean, valueTable: boolean) {
    this.keys = keyTable ? new Map() : null;
    this.values = valueTable ? new Map() : null;
  }

  ensure(n: number): void {
    if (this.pos + n <= this.buf.length) {
      return;
    }
    let size = this.buf.length * 2;
    while (size < this.pos + n) {
      size *= 2;
    }
    const next = new Uint8Array(size);
    // Copy everything, not just up to `pos`: strings are encoded ahead of their header.
    next.set(this.buf);
    this.buf = next;
    this.view = new DataView(next.buffer);
  }

  u8(byte: number): void {
    this.ensure(1);
    this.buf[this.pos++] = byte;
  }

  varint(value: number): void {
    this.ensure(5);
    while (value >= 0x80) {
      this.buf[this.pos++] = (value & 0x7f) | 0x80;
      value >>>= 7;
    }
    this.buf[this.pos++] = value;
  }

  /** A varint of up to 53 bits, past what `>>>` can shift. */
  varint64(value: number): void {
    this.ensure(8);
    while (value >= 0x80) {
      this.buf[this.pos++] = (value % 0x80) | 0x80;
      value = Math.floor(value / 0x80);
    }
    this.buf[this.pos++] = value;
  }

  header(short: number, long: number, len: number): void {
    if (len <= SHORT_MAX) {
      this.ensure(2);
      this.buf[this.pos++] = short;
      this.buf[this.pos++] = len;
    } else {
      this.u8(long);
      this.varint(len);
    }
  }

  /**
   * Encode `str` as UTF-8 at `pos + reserve`, leaving `pos` unchanged, and return its byte length.
   * Leaves room for the length prefix to grow past `reserve` bytes.
   */
  encodeUtf8(str: string, reserve: number): number {
    const max = str.length * 3;
    this.ensure(reserve + max + 5);
    const start = this.pos + reserve;
    // Short ASCII strings are cheaper to write by hand than through `TextEncoder`.
    if (str.length <= 32) {
      let i = 0;
      for (; i < str.length; i++) {
        const code = str.charCodeAt(i);
        if (code > 0x7f) {
          break;
        }
        this.buf[start + i] = code;
      }
      if (i === str.length) {
        return i;
      }
    }
    return textEncoder.encodeInto(str, this.buf.subarray(start, start + max)).written;
  }

  /** Move `len` bytes encoded at `pos + reserve` so they follow a `headerLen`-byte prefix. */
  place(reserve: number, headerLen: number, len: number): void {
    if (headerLen === reserve) {
      return;
    }
    const start = this.pos + reserve;
    this.buf.copyWithin(this.pos + headerLen, start, start + len);
  }

  string(str: string): void {
    if (this.values !== null) {
      this.u8(ValueType.InternedString);
      this.varint(intern(this.values, str));
      return;
    }
    const len = this.encodeUtf8(str, 2);
    this.place(2, len <= SHORT_MAX ? 2 : 1 + varintLen(len), len);
    this.header(ValueType.ShortStringUtf8, ValueType.StringUtf8, len);
    this.pos += len;
  }

  key(key: string): void {
    if (this.keys === null) {
      const len = this.encodeUtf8(key, 1);
      this.place(1, varintLen(len), len);
      this.varint(len);
      this.pos += len;
      return;
    }
    this.varint(intern(this.keys, key));
  }

  value(value: unknown): void {
    switch (typeof value) {
      case "boolean":
        this.ensure(2);
        this.buf[this.pos++] = ValueType.Bool;
        this.buf[this.pos++] = value ? 1 : 0;
        return;
      case "number":
        return this.number(value);
      case "bigint":
        return this.bigint(value);
      case "string":
        return this.string(value);
      case "object":
        if (value === null) {
          return this.u8(ValueType.Null);
        }
        return this.object(value);
      default:
        throw new SpectaBinaryError(`cannot encode ${typeof value}`);
    }
  }

  number(value: number): void {
    // `-0` is a safe integer, written as `0` like `JSON.stringify` does.
    if (Number.isSafeInteger(value)) {
      if (value >= -0x8000 && value <= 0x7fff) {
        this.ensure(3);
        this.buf[this.pos] = ValueType.Int16;
        this.view.setInt16(this.pos + 1, value, true);
        this.pos += 3;
      } else if (value >= -0x8000_0000 && value <= 0x7fff_ffff) {
        this.ensure(5);
        this.buf[this.pos] = ValueType.Int32;
        this.view.setInt32(this.pos + 1, value, true);
        this.pos += 5;
      } else {
        const hi = Math.floor(value / TWO_POW_32);
        this.ensure(9);
        this.buf[this.pos] = ValueType.Int64;
        this.view.setUint32(this.pos + 1, value - hi * TWO_POW_32, true);
        this.view.setInt32(this.pos + 5, hi, true);
        this.pos += 9;
      }
      return;
    }
    const decimal = Number.isFinite(value) ? shortestDecimal(value) : null;
    if (decimal && 1 + varintLen(decimal.head) + varintLen(decimal.mantissa) <= FLOAT64_LEN) {
      this.u8(ValueType.Decimal);
      this.varint(decimal.head);
      this.varint64(decimal.mantissa);
      return;
    }
    this.ensure(9);
    this.buf[this.pos] = ValueType.Float64;
    this.view.setFloat64(this.pos + 1, value, true);
    this.pos += 9;
  }

  bigint(value: bigint): void {
    if (value >= -0x8000_0000n && value <= 0x7fff_ffffn) {
      return this.number(Number(value));
    }
    if (value < I64_MIN || value > I64_MAX) {
      throw new SpectaBinaryError(`bigint ${value} does not fit in i64`);
    }
    this.ensure(9);
    this.buf[this.pos] = ValueType.Int64;
    this.view.setBigInt64(this.pos + 1, value, true);
    this.pos += 9;
  }

  object(value: object): void {
    if (value instanceof Uint8Array) {
      this.header(ValueType.ShortByteArray, ValueType.ByteArray, value.byteLength);
      this.ensure(value.byteLength);
      this.buf.set(value, this.pos);
      this.pos += value.byteLength;
      return;
    }
    if (
      value instanceof Map ||
      value instanceof Set ||
      value instanceof ArrayBuffer ||
      ArrayBuffer.isView(value)
    ) {
      throw new SpectaBinaryError(`cannot encode ${value.constructor.name}`);
    }
    if (this.stack.has(value)) {
      throw new SpectaBinaryError("cannot encode a cyclic structure");
    }
    this.stack.add(value);
    if (Array.isArray(value)) {
      this.array(value);
    } else {
      this.record(value as Record<string, unknown>);
    }
    this.stack.delete(value);
  }

  array(array: unknown[]): void {
    this.header(ValueType.ShortArray, ValueType.Array, array.length);
    let i = 0;
    try {
      for (; i < array.length; i++) {
        const item = resolve(array[i], i);
        if (item === undefined) {
          this.u8(ValueType.Null);
        } else {
          this.value(item);
        }
      }
    } catch (err) {
      throw prepend(err, `[${i}]`);
    }
  }

  record(record: Record<string, unknown>): void {
    // The count isn't known until `undefined` properties are skipped, so write a short
    // header and patch it afterwards.
    const at = this.pos;
    this.header(ValueType.ShortObject, ValueType.Object, 0);
    let count = 0;
    let key = "";
    try {
      for (key of Object.keys(record)) {
        const item = resolve(record[key], key);
        if (item === undefined) {
          continue;
        }
        this.key(key);
        this.value(item);
        count++;
      }
    } catch (err) {
      throw prepend(err, keySegment(key));
    }
    if (count <= SHORT_MAX) {
      this.buf[at + 1] = count;
      return;
    }
    const grow = varintLen(count) - 1;
    this.ensure(grow);
    this.buf.copyWithin(at + 2 + grow, at + 2, this.pos);
    const end = this.pos + grow;
    this.pos = at;
    this.header(ValueType.ShortObject, ValueType.Object, count);
    this.pos = end;
  }

  finish(): Uint8Array {
    // Maps iterate in insertion order, which is id order. Empty tables are never written.
    const encodeTable = (table: Map<string, number> | null) =>
      table !== null && table.size > 0 ? Array.from(table.keys(), (s) => textEncoder.encode(s)) : null;
    const keys = encodeTable(this.keys);
    const values = encodeTable(this.values);
    const tables = [keys, values].filter((table) => table !== null);
    let size = 4 + this.pos;
    for (const table of tables) {
      size += varintLen(table.length);
      for (const s of table) {
        size += varintLen(s.length) + s.length;
      }
    }
    const out = new Writer(false, false);
    out.buf = new Uint8Array(size);
    out.view = new DataView(out.buf.buffer);
    const flags = (keys ? FLAG_HAS_KEY_TABLE : 0) | (values ? FLAG_HAS_VALUE_TABLE : 0);
    out.buf.set([0x53, 0x42, VERSION, flags]);
    out.pos = 4;
    for (const table of tables) {
      out.varint(table.length);
      for (const s of table) {
        out.varint(s.length);
        out.buf.set(s, out.pos);
        out.pos += s.length;
      }
    }
    out.buf.set(this.buf.subarray(0, this.pos), out.pos);
    return out.buf;
  }
}

/** The id of `s` in `table`, adding it with the next id if it is new. */
function intern(table: Map<string, number>, s: string): number {
  let id = table.get(s);
  if (id === undefined) {
    id = table.size;
    table.set(s, id);
  }
  return id;
}
