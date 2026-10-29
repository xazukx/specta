// Malformed input must throw `SpectaBinaryError`, never another error, a hang or prototype
// pollution. Also covers the encoder's JSON-like rules and the decode options.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  type DecodeOptions,
  SpectaBinaryError,
  ValueType,
  decode,
  encode,
} from "../../specta-binary/runtime/specta_binary.ts";

const bytes = (hex: string) =>
  new Uint8Array(hex.split(/\s+/).filter(Boolean).map((byte) => Number.parseInt(byte, 16)));
/** A document with no key table around a hand-written body. */
const doc = (body: string) => bytes(`53 42 01 00 ${body}`);
const hex = (data: Uint8Array) =>
  Array.from(data, (byte) => byte.toString(16).padStart(2, "0").toUpperCase()).join(" ");

function fails(input: Uint8Array, reason: RegExp, options?: DecodeOptions): SpectaBinaryError {
  try {
    decode(input, options);
  } catch (err) {
    assert.ok(err instanceof SpectaBinaryError, `expected SpectaBinaryError, got ${err}`);
    assert.match(err.reason, reason);
    return err;
  }
  assert.fail(`decoded ${hex(input)}`);
}

const sample = {
  users: [
    { name: "ü0", email: null, age: 0 },
    { name: "ü1", email: "a@b.c", age: 1 },
  ],
  meta: { v: [1.5, -1, "x", null, true, 2n ** 60n], long: "y".repeat(300) },
  blob: new Uint8Array(300).fill(9),
};
const samples = [encode(sample), encode(sample, { keyTable: false })];

test("truncated buffers", () => {
  for (const full of samples) {
    for (let len = 0; len < full.length; len++) {
      fails(full.subarray(0, len), /./);
    }
  }
});

test("bad header", () => {
  fails(bytes(""), /unexpected end of input/);
  fails(bytes("53 43 01 00 00"), /invalid magic bytes/);
  fails(bytes("53 42 02 00 00"), /unsupported version 2/);
  fails(bytes("53 42 01 02 00"), /unknown header flags 0x02/);
  fails(bytes("53 42 01 00"), /unexpected end of input/);
  const err = fails(bytes("53 42 01 80 00"), /unknown header flags 0x80/);
  assert.equal(err.path, "");
  assert.equal(err.offset, 3);
});

test("unknown tags", () => {
  const known = new Set<number>(Object.values(ValueType));
  for (let tag = 0; tag < 256; tag++) {
    if (!known.has(tag)) fails(doc(tag.toString(16).padStart(2, "0")), /unknown value type tag/);
  }
  // Same message as the Rust reader.
  const err = fails(doc("37 02 00 01"), /unknown value type tag 0x01/);
  assert.equal(err.message, "at $[1]: unknown value type tag 0x01 (offset 7)");
});

test("invalid bool", () => {
  fails(doc("05 02"), /invalid bool byte 0x02/);
});

test("huge lengths", () => {
  const huge = "FF FF FF FF 0F";
  for (const tag of ["1E", "28", "32", "3C"]) {
    fails(doc(`${tag} ${huge}`), /length 4294967295 exceeds the 0 remaining bytes/);
  }
  fails(bytes(`53 42 01 01 ${huge} 00`), /length 4294967295 exceeds/);
  fails(bytes(`53 42 01 01 01 ${huge}`), /length 4294967295 exceeds/);
  fails(doc("37 05 00"), /length 5 exceeds the 1 remaining bytes/);
});

test("malformed varints", () => {
  fails(doc("1E 80 80 80 80 80 00"), /varint is longer than 5 bytes/);
  fails(doc("1E FF FF FF FF 1F"), /varint is longer than 5 bytes/);
  fails(doc("1E 80 80"), /unexpected end of input/);
  assert.equal(decode(doc("1E 81 00 61")), "a");
});

test("key id out of range", () => {
  fails(bytes("53 42 01 01 01 01 61 41 01 01 00"), /key id 1 is out of range for a key table of 1 keys/);
});

test("depth limit", () => {
  const nested = (depth: number) => doc(`${"37 01 ".repeat(depth)}00`);
  decode(nested(128));
  fails(nested(129), /maximum nesting depth of 128 exceeded/);
  fails(nested(100_000), /maximum nesting depth of 128 exceeded/);
  const err = fails(nested(3), /maximum nesting depth of 2/, { maxDepth: 2 });
  assert.equal(err.message, "at $[0][0]: maximum nesting depth of 2 exceeded (offset 8)");
  fails(doc(`${"41 01 01 61 ".repeat(129)}00`), /maximum nesting depth/);
});

test("invalid UTF-8", () => {
  fails(doc("23 02 C3 28"), /invalid UTF-8/);
  fails(doc("41 01 01 FF 00"), /invalid UTF-8/);
  fails(bytes("53 42 01 01 01 01 FF 41 01 00 00"), /invalid UTF-8/);
  fails(doc("23 02 C0 AF"), /invalid UTF-8/);
  fails(doc("23 03 ED A0 80"), /invalid UTF-8/);
  // The long-string path goes through `TextDecoder` rather than the ASCII fast path.
  fails(doc(`23 20 ${"61 ".repeat(31)}FF`), /invalid UTF-8/);
});

test("trailing bytes", () => {
  fails(doc("00 00"), /trailing bytes after the root value/);
});

test("__proto__ keys don't pollute prototypes", () => {
  for (const keyTable of [true, false]) {
    const input = encode(JSON.parse('{"__proto__": {"polluted": true}, "a": 1}'), { keyTable });
    const value = decode<Record<string, unknown>>(input);
    assert.equal(Object.getPrototypeOf(value), Object.prototype);
    assert.deepEqual(Object.keys(value), ["__proto__", "a"]);
    assert.deepEqual(Object.getOwnPropertyDescriptor(value, "__proto__")?.value, { polluted: true });
    assert.equal(value.polluted, undefined);
    assert.equal(({} as Record<string, unknown>).polluted, undefined);
  }
});

test("mutations only ever throw SpectaBinaryError", () => {
  let state = 0x2545f491;
  const next = () => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return state >>> 0;
  };
  for (const original of samples) {
    for (let i = 0; i < 5_000; i++) {
      const mutated = original.slice();
      for (let n = next() % 4; n >= 0; n--) mutated[next() % mutated.length] = next() & 0xff;
      try {
        decode(mutated);
      } catch (err) {
        assert.ok(err instanceof SpectaBinaryError, String(err));
      }
    }
  }
});

test("error paths", () => {
  const err = fails(
    encode({ users: [{ email: null }, { email: null }, { email: "x￿" }] }).map((byte) =>
      byte === 0xef ? 0xff : byte,
    ),
    /invalid UTF-8/,
  );
  assert.equal(err.path, '$.users[2].email');
  assert.equal(err.message, `at $.users[2].email: invalid UTF-8 (offset ${err.offset})`);

  const badBool = encode({ "a b": [0, true] });
  badBool[badBool.length - 1] = 2;
  const quoted = fails(badBool, /invalid bool/);
  assert.equal(quoted.path, '$["a b"][1]');
});

test("encode follows JSON.stringify", () => {
  assert.deepEqual(decode(encode({ a: undefined, b: 1 })), { b: 1 });
  assert.deepEqual(decode(encode([undefined, 1])), [null, 1]);
  assert.equal(hex(encode(-0)), "53 42 01 00 0A 00 00 00 00");
  assert.equal(decode(encode(new Date(0))), "1970-01-01T00:00:00.000Z");
  assert.deepEqual(decode(encode({ d: { toJSON: () => undefined }, e: 1 })), { e: 1 });
  assert.equal(encode(NaN)[4], ValueType.Float64);
  assert.equal(encode(2 ** 53)[4], ValueType.Float64);
  assert.equal(encode(2 ** 53 - 1)[4], ValueType.Int64);
  assert.equal(encode(-(2 ** 31))[4], ValueType.Int32);
  assert.equal(encode(-(2 ** 31) - 1)[4], ValueType.Int64);
  assert.equal(encode(0.5)[4], ValueType.Float64);
  assert.equal(encode(5n)[4], ValueType.Int32);
  assert.equal(encode(-(2n ** 63n))[4], ValueType.Int64);
  // Writers must pick the short form for lengths up to 255.
  assert.equal(encode("é".repeat(127))[4], ValueType.ShortStringUtf8);
  assert.equal(encode("é".repeat(128))[4], ValueType.StringUtf8);
  assert.equal(encode(new Uint8Array(255))[4], ValueType.ShortByteArray);
  assert.equal(encode(new Uint8Array(256))[4], ValueType.ByteArray);
  // An object with more than 255 keys, some of them `undefined`, is patched to the long form.
  const record: Record<string, unknown> = {};
  for (let i = 0; i < 300; i++) record[`k${i}`] = i % 2 ? undefined : i;
  assert.equal(encode(record, { keyTable: false })[4], ValueType.ShortObject);
  for (let i = 0; i < 300; i++) record[`k${i}`] = i;
  const long = encode(record, { keyTable: false });
  assert.equal(hex(long.subarray(4, 7)), "3C AC 02");
  assert.deepEqual(decode(long), record);
});

test("encode rejects unsupported values", () => {
  const rejects = (value: unknown, message: string) =>
    assert.throws(
      () => encode(value),
      (err) => err instanceof SpectaBinaryError && err.message === message,
    );
  rejects(undefined, "at $: cannot encode undefined");
  rejects({ a: [1, new Map()] }, "at $.a[1]: cannot encode Map");
  rejects({ s: new Set() }, "at $.s: cannot encode Set");
  rejects({ f: () => 1 }, "at $.f: cannot encode function");
  rejects([Symbol("s")], "at $[0]: cannot encode symbol");
  rejects({ t: new Int16Array(1) }, "at $.t: cannot encode Int16Array");
  rejects({ big: 2n ** 63n }, "at $.big: bigint 9223372036854775808 does not fit in i64");
  const cyclic: Record<string, unknown> = { a: {} };
  (cyclic.a as Record<string, unknown>).self = cyclic;
  rejects(cyclic, "at $.a.self: cannot encode a cyclic structure");
  // The same object twice is fine when it isn't a cycle.
  const shared = { x: 1 };
  assert.deepEqual(decode(encode([shared, shared])), [shared, shared]);
});

test("decode options", () => {
  const big = encode(2n ** 60n);
  assert.equal(decode(big), 2n ** 60n);
  assert.equal(decode(big, { int64: "number" }), 2 ** 60);
  assert.equal(decode(encode(2 ** 40), { int64: "bigint" }), 2n ** 40n);
  assert.equal(decode(encode(-(2 ** 40))), -(2 ** 40));
  // "auto" switches to `bigint` just outside `Number.MAX_SAFE_INTEGER` (2^53 - 1).
  assert.equal(decode(doc("0F FF FF FF FF FF FF 1F 00")), 2 ** 53 - 1);
  assert.equal(decode(doc("0F 01 00 00 00 00 00 E0 FF")), -(2 ** 53 - 1));
  assert.equal(decode(doc("0F 00 00 00 00 00 00 20 00")), 2n ** 53n);
  assert.equal(decode(doc("0F 00 00 00 00 00 00 E0 FF")), -(2n ** 53n));

  const f32 = doc("14 CD CC CC 3D"); // 0.1f32
  assert.equal(decode(f32), 0.1);
  assert.equal(decode(f32, { float32: "exact" }), Math.fround(0.1));
  assert.ok(Object.is(decode(doc("14 00 00 00 80")), -0));
  assert.ok(Number.isNaN(decode(doc("14 00 00 C0 7F"))));

  const blob = doc("2D 03 01 02 03");
  const view = decode<Uint8Array>(blob);
  assert.ok(view instanceof Uint8Array);
  assert.equal(view.buffer, blob.buffer, "a view into the input, not a copy");
  assert.deepEqual(decode(blob, { bytes: "array" }), [1, 2, 3]);
});
