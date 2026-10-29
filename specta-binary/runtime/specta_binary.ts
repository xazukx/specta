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
 * - `float32`: `"shortest"` (default) gives the shortest decimal that rounds to the same
 *   `f32`, matching `serde_json` (`0.1f32` decodes to `0.1`). `"exact"` is faster.
 * - `maxDepth`: the maximum nesting of arrays and objects. Defaults to 128.
 *
 * Encoding (`EncodeOptions`) mirrors `JSON.stringify`, except that unsupported values throw:
 * - `keyTable`: intern object keys into a table at the start of the document (default `true`).
 * - `undefined` object properties are omitted, `undefined` array items become `null`.
 * - Integer numbers are written as `Int32`/`Int64`, other numbers (including `NaN` and
 *   `±Infinity`) as `Float64`. `bigint` is written as an integer and must fit in an `i64`.
 * - `Uint8Array` is written as a byte array. Objects with a `toJSON()` method (like `Date`)
 *   are encoded as its result.
 * - `Map`, `Set`, functions, symbols, other typed arrays and cycles throw.
 *
 * Errors are thrown as `SpectaBinaryError`, which carries the `path` to the failing value
 * (like `$.users[3].email`) and the byte `offset` where it starts (`-1` when encoding).
 */

export const ValueType = {
  Null: 0,
  Bool: 5,
  Int32: 10,
  Int64: 15,
  Float32: 20,
  Float64: 25,
  StringUtf8: 30,
  ShortStringUtf8: 35,
  ByteArray: 40,
  ShortByteArray: 45,
  Array: 50,
  ShortArray: 55,
  Object: 60,
  ShortObject: 65,
} as const;

export type ValueType = (typeof ValueType)[keyof typeof ValueType];

export interface EncodeOptions {
  /** Intern object keys into a key table. Defaults to `true`. */
  keyTable?: boolean;
}

export interface DecodeOptions {
  /** How to return `Int64` values. Defaults to `"auto"`. */
  int64?: "auto" | "number" | "bigint";
  /** How to return byte arrays. Defaults to `"uint8array"`. */
  bytes?: "uint8array" | "array";
  /** How to return `Float32` values. Defaults to `"shortest"`. */
  float32?: "shortest" | "exact";
  /** The maximum nesting depth of arrays and objects. Defaults to `128`. */
  maxDepth?: number;
}

const VERSION = 0x01;
const FLAG_HAS_KEY_TABLE = 0x01;
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
  depth = 0;
  readonly int64: "auto" | "number" | "bigint";
  readonly bytesAsArray: boolean;
  readonly exactFloat32: boolean;
  readonly maxDepth: number;

  constructor(bytes: Uint8Array, options: DecodeOptions) {
    this.bytes = bytes;
    this.view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    this.end = bytes.byteLength;
    this.int64 = options.int64 ?? "auto";
    this.bytesAsArray = options.bytes === "array";
    this.exactFloat32 = options.float32 === "exact";
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
    if ((flags & ~FLAG_HAS_KEY_TABLE) !== 0) {
      throw new SpectaBinaryError(`unknown header flags ${hex(flags)}`, 3);
    }
    if (flags & FLAG_HAS_KEY_TABLE) {
      const count = this.varLen();
      const keys: string[] = [];
      for (let i = 0; i < count; i++) keys.push(this.str(this.varLen()));
      this.keys = keys;
    }
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
      if (i === 4 && byte > 0x0f) break;
      value += (byte & 0x7f) * 2 ** (7 * i);
      if ((byte & 0x80) === 0) return value;
    }
    throw new SpectaBinaryError("varint is longer than 5 bytes or overflows 32 bits", at);
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
        if (byte > 0x7f) return this.utf8(start, len);
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
    if (this.keys === null) return this.str(this.len(at, n));
    const key = this.keys[n];
    if (key === undefined) {
      throw new SpectaBinaryError(
        `key id ${n} is out of range for a key table of ${this.keys.length} keys`,
        at,
      );
    }
    return key;
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
        if (byte > 1) throw new SpectaBinaryError(`invalid bool byte ${hex(byte)}`, this.pos - 1);
        return byte === 1;
      }
      case ValueType.Int32: {
        this.need(4);
        const value = this.view.getInt32(this.pos, true);
        this.pos += 4;
        return value;
      }
      case ValueType.Int64:
        return this.int64Value();
      case ValueType.Float32:
        return this.float32Value();
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
    if (this.int64 === "bigint") return this.view.getBigInt64(at, true);
    // `hi * 2^32 + lo` is exact for safe integers and correctly rounded otherwise.
    const lo = this.view.getUint32(at, true);
    const hi = this.view.getInt32(at + 4, true);
    const value = hi * TWO_POW_32 + lo;
    if (this.int64 === "number" || Number.isSafeInteger(value)) return value;
    return this.view.getBigInt64(at, true);
  }

  float32Value(): number {
    this.need(4);
    const value = this.view.getFloat32(this.pos, true);
    this.pos += 4;
    // `0` is checked first so `-0` keeps its sign.
    if (this.exactFloat32 || value === 0 || !Number.isFinite(value)) return value;
    for (let precision = 1; precision <= 9; precision++) {
      const shortest = Number(value.toPrecision(precision));
      if (Math.fround(shortest) === value) return shortest;
    }
    return value;
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
      for (; i < count; i++) out.push(this.value());
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
  const writer = new Writer(options?.keyTable ?? true);
  try {
    const root = resolve(value, "");
    if (root === undefined) throw new SpectaBinaryError("cannot encode undefined");
    writer.value(root);
  } catch (err) {
    throw rooted(err);
  }
  return writer.finish();
}

/** Apply `toJSON`, like `JSON.stringify` does. */
function resolve(value: unknown, key: string | number): unknown {
  if (typeof value === "object" && value !== null) {
    const toJSON = (value as { toJSON?: unknown }).toJSON;
    if (typeof toJSON === "function") return toJSON.call(value, String(key));
  }
  return value;
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
  /** The arrays and objects currently being encoded, to detect cycles. */
  readonly stack = new Set<object>();

  constructor(keyTable: boolean) {
    this.keys = keyTable ? new Map() : null;
  }

  ensure(n: number): void {
    if (this.pos + n <= this.buf.length) return;
    let size = this.buf.length * 2;
    while (size < this.pos + n) size *= 2;
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
        if (code > 0x7f) break;
        this.buf[start + i] = code;
      }
      if (i === str.length) return i;
    }
    return textEncoder.encodeInto(str, this.buf.subarray(start, start + max)).written;
  }

  /** Move `len` bytes encoded at `pos + reserve` so they follow a `headerLen`-byte prefix. */
  place(reserve: number, headerLen: number, len: number): void {
    if (headerLen === reserve) return;
    const start = this.pos + reserve;
    this.buf.copyWithin(this.pos + headerLen, start, start + len);
  }

  string(str: string): void {
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
    let id = this.keys.get(key);
    if (id === undefined) {
      id = this.keys.size;
      this.keys.set(key, id);
    }
    this.varint(id);
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
        if (value === null) return this.u8(ValueType.Null);
        return this.object(value);
      default:
        throw new SpectaBinaryError(`cannot encode ${typeof value}`);
    }
  }

  number(value: number): void {
    if (Number.isInteger(value)) {
      if (value >= -0x8000_0000 && value <= 0x7fff_ffff) {
        this.ensure(5);
        this.buf[this.pos] = ValueType.Int32;
        // `-0` is written as `0`, like `JSON.stringify`.
        this.view.setInt32(this.pos + 1, value, true);
        this.pos += 5;
        return;
      }
      if (Number.isSafeInteger(value)) {
        const hi = Math.floor(value / TWO_POW_32);
        this.ensure(9);
        this.buf[this.pos] = ValueType.Int64;
        this.view.setUint32(this.pos + 1, value - hi * TWO_POW_32, true);
        this.view.setInt32(this.pos + 5, hi, true);
        this.pos += 9;
        return;
      }
    }
    this.ensure(9);
    this.buf[this.pos] = ValueType.Float64;
    this.view.setFloat64(this.pos + 1, value, true);
    this.pos += 9;
  }

  bigint(value: bigint): void {
    if (value >= -0x8000_0000n && value <= 0x7fff_ffffn) return this.number(Number(value));
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
    if (this.stack.has(value)) throw new SpectaBinaryError("cannot encode a cyclic structure");
    this.stack.add(value);
    if (Array.isArray(value)) this.array(value);
    else this.record(value as Record<string, unknown>);
    this.stack.delete(value);
  }

  array(array: unknown[]): void {
    this.header(ValueType.ShortArray, ValueType.Array, array.length);
    let i = 0;
    try {
      for (; i < array.length; i++) {
        const item = resolve(array[i], i);
        if (item === undefined) this.u8(ValueType.Null);
        else this.value(item);
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
        if (item === undefined) continue;
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
    const keys =
      this.keys !== null && this.keys.size > 0
        ? Array.from(this.keys.keys(), (key) => textEncoder.encode(key))
        : null;
    let size = 4 + this.pos;
    if (keys !== null) {
      size += varintLen(keys.length);
      for (const key of keys) size += varintLen(key.length) + key.length;
    }
    const out = new Writer(false);
    out.buf = new Uint8Array(size);
    out.view = new DataView(out.buf.buffer);
    out.buf.set([0x53, 0x42, VERSION, keys === null ? 0 : FLAG_HAS_KEY_TABLE]);
    out.pos = 4;
    if (keys !== null) {
      out.varint(keys.length);
      for (const key of keys) {
        out.varint(key.length);
        out.buf.set(key, out.pos);
        out.pos += key.length;
      }
    }
    out.buf.set(this.buf.subarray(0, this.pos), out.pos);
    return out.buf;
  }
}
