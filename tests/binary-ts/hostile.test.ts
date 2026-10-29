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
/** A document with no tables around a hand-written body. */
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
const samples = [
  encode(sample),
  encode(sample, { keyTable: false }),
  encode(sample, { valueTable: false }),
  encode(sample, { keyTable: false, valueTable: false }),
];

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
  fails(bytes("53 42 01 04 00"), /unknown header flags 0x04/);
  fails(bytes("53 42 01 00"), /unexpected end of input/);
  const err = fails(bytes("53 42 01 83 00"), /unknown header flags 0x83/);
  assert.equal(err.path, "");
  assert.equal(err.offset, 3);
});

test("unknown tags", () => {
  const known = new Set<number>(Object.values(ValueType));
  for (let tag = 0; tag < 256; tag++) {
    if (!known.has(tag)) {
      fails(doc(tag.toString(16).padStart(2, "0")), /unknown value type tag/);
    }
  }
  // Same message as the Rust reader.
  const err = fails(doc("41 02 00 01"), /unknown value type tag 0x01/);
  assert.equal(err.message, "at $[1]: unknown value type tag 0x01 (offset 7)");
});

test("invalid bool", () => {
  fails(doc("05 02"), /invalid bool byte 0x02/);
});

test("huge lengths", () => {
  const huge = "FF FF FF FF 0F";
  for (const tag of ["23", "32", "3C", "46"]) {
    fails(doc(`${tag} ${huge}`), /length 4294967295 exceeds the 0 remaining bytes/);
  }
  // Table counts and string lengths, in the key table and the value table.
  for (const flags of ["01", "02"]) {
    fails(bytes(`53 42 01 ${flags} ${huge} 00`), /length 4294967295 exceeds/);
    fails(bytes(`53 42 01 ${flags} 01 ${huge}`), /length 4294967295 exceeds/);
  }
  fails(doc("41 05 00"), /length 5 exceeds the 1 remaining bytes/);
});

test("malformed varints", () => {
  fails(doc("23 80 80 80 80 80 00"), /varint is too long or overflows/);
  fails(doc("23 FF FF FF FF 1F"), /varint is too long or overflows/);
  fails(doc("23 80 80"), /unexpected end of input/);
  assert.equal(decode(doc("23 81 00 61")), "a");
  // A decimal's head is a varint and its mantissa a varint64 (at most 10 bytes, 64 bits).
  fails(doc("19 FF FF FF FF 1F 01"), /varint is too long or overflows/);
  fails(doc("19 02 FF FF FF FF FF FF FF FF FF 02"), /varint is too long or overflows/);
  fails(doc("19 02 80 80 80 80 80 80 80 80 80 80 00"), /varint is too long or overflows/);
  for (const body of ["19", "19 02", "19 02 80", "1E 00 00 00 00 00 00 F8"]) {
    fails(doc(body), /unexpected end of input/);
  }
});

test("key id out of range", () => {
  fails(bytes("53 42 01 01 01 01 61 4B 01 01 00"), /key id 1 is out of range for a key table of 1 keys/);
});

test("value id out of range", () => {
  fails(bytes("53 42 01 02 01 01 61 2D 01"), /value id 1 is out of range for a value table of 1 values/);
  const err = fails(doc("2D 00"), /value id 0 is out of range for a value table of 0 values/);
  assert.equal(err.offset, 5);
});

test("extreme decimals saturate", () => {
  // The largest mantissa, with the largest and smallest exponents (+/- 2^30) and both signs.
  const max = "FF FF FF FF FF FF FF FF FF 01";
  assert.equal(decode(doc(`19 FC FF FF FF 0F ${max}`)), Infinity);
  assert.equal(decode(doc(`19 FD FF FF FF 0F ${max}`)), -Infinity);
  assert.ok(Object.is(decode(doc(`19 FF FF FF FF 0F ${max}`)), -0));
});

test("depth limit", () => {
  const nested = (depth: number) => doc(`${"41 01 ".repeat(depth)}00`);
  decode(nested(128));
  fails(nested(129), /maximum nesting depth of 128 exceeded/);
  fails(nested(100_000), /maximum nesting depth of 128 exceeded/);
  const err = fails(nested(3), /maximum nesting depth of 2/, { maxDepth: 2 });
  assert.equal(err.message, "at $[0][0]: maximum nesting depth of 2 exceeded (offset 8)");
  fails(doc(`${"4B 01 01 61 ".repeat(129)}00`), /maximum nesting depth/);
});

test("invalid UTF-8", () => {
  fails(doc("28 02 C3 28"), /invalid UTF-8/);
  fails(doc("4B 01 01 FF 00"), /invalid UTF-8/);
  fails(bytes("53 42 01 01 01 01 FF 4B 01 00 00"), /invalid UTF-8/);
  fails(bytes("53 42 01 02 01 01 FF 2D 00"), /invalid UTF-8/);
  fails(doc("28 02 C0 AF"), /invalid UTF-8/);
  fails(doc("28 03 ED A0 80"), /invalid UTF-8/);
  // The long-string path goes through `TextDecoder` rather than the ASCII fast path.
  fails(doc(`28 20 ${"61 ".repeat(31)}FF`), /invalid UTF-8/);
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
      for (let n = next() % 4; n >= 0; n--) {
        mutated[next() % mutated.length] = next() & 0xff;
      }
      try {
        decode(mutated);
      } catch (err) {
        assert.ok(err instanceof SpectaBinaryError, String(err));
      }
    }
  }
});

test("error paths", () => {
  // Inline strings, so the bad UTF-8 is in the value rather than in the value table.
  const users = { users: [{ email: null }, { email: null }, { email: "x￿" }] };
  const err = fails(
    encode(users, { valueTable: false }).map((byte) => (byte === 0xef ? 0xff : byte)),
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
  assert.equal(hex(encode(-0)), "53 42 01 00 0A 00 00");
  assert.equal(decode(encode(new Date(0))), "1970-01-01T00:00:00.000Z");
  assert.deepEqual(decode(encode({ d: { toJSON: () => undefined }, e: 1 })), { e: 1 });
  // Safe integers use the narrowest integer tag.
  assert.equal(encode(-(2 ** 15))[4], ValueType.Int16);
  assert.equal(encode(2 ** 15 - 1)[4], ValueType.Int16);
  assert.equal(encode(2 ** 15)[4], ValueType.Int32);
  assert.equal(encode(-(2 ** 31))[4], ValueType.Int32);
  assert.equal(encode(-(2 ** 31) - 1)[4], ValueType.Int64);
  assert.equal(encode(2 ** 53 - 1)[4], ValueType.Int64);
  assert.equal(encode(5n)[4], ValueType.Int16);
  assert.equal(encode(-(2n ** 63n))[4], ValueType.Int64);
  // Everything else is a `Decimal` (head: zigzag exponent and sign, then the mantissa) when
  // that takes at most 9 bytes, and a `Float64` otherwise.
  assert.equal(hex(encode(1.5)), "53 42 01 00 19 02 0F");
  assert.equal(hex(encode(-1.5)), "53 42 01 00 19 03 0F");
  assert.equal(hex(encode(19.99)), "53 42 01 00 19 06 CF 0F");
  assert.equal(hex(encode(1e21)), "53 42 01 00 19 54 01");
  assert.equal(hex(encode(1e300)), "53 42 01 00 19 B0 09 01");
  assert.equal(hex(encode(0.1 + 0.2)), "53 42 01 00 1E 34 33 33 33 33 33 D3 3F");
  assert.equal(hex(encode(NaN)), "53 42 01 00 1E 00 00 00 00 00 00 F8 7F");
  assert.equal(hex(encode(-Infinity)), "53 42 01 00 1E 00 00 00 00 00 00 F0 FF");
  assert.equal(encode(2 ** 53)[4], ValueType.Float64);
  assert.equal(hex(encode(0.123456789012345)), "53 42 01 00 19 3A F9 BE B7 B0 88 89 1C");
  // Repeated strings share a value table entry, unless the table is off.
  assert.equal(hex(encode(["a", "b", "a"])), "53 42 01 02 02 01 61 01 62 41 03 2D 00 2D 01 2D 00");
  assert.equal(hex(encode(["a", "a"], { valueTable: false })), "53 42 01 00 41 02 28 01 61 28 01 61");
  // Writers must pick the short form for lengths up to 255.
  const inline = { valueTable: false };
  assert.equal(encode("é".repeat(127), inline)[4], ValueType.ShortStringUtf8);
  assert.equal(encode("é".repeat(128), inline)[4], ValueType.StringUtf8);
  assert.equal(encode(new Uint8Array(255))[4], ValueType.ShortByteArray);
  assert.equal(encode(new Uint8Array(256))[4], ValueType.ByteArray);
  // `Buffer.toJSON()` would give `{ type: "Buffer", data: number[] }`, 3 bytes per byte.
  assert.equal(hex(encode(Buffer.from([1, 2]))), "53 42 01 00 37 02 01 02");
  assert.deepEqual(decode(encode({ b: Buffer.from("hi") }), { bytes: "array" }), { b: [104, 105] });
  // An object with more than 255 keys, some of them `undefined`, is patched to the long form.
  const record: Record<string, unknown> = {};
  for (let i = 0; i < 300; i++) {
    record[`k${i}`] = i % 2 ? undefined : i;
  }
  assert.equal(encode(record, { keyTable: false })[4], ValueType.ShortObject);
  for (let i = 0; i < 300; i++) {
    record[`k${i}`] = i;
  }
  const long = encode(record, { keyTable: false });
  assert.equal(hex(long.subarray(4, 7)), "46 AC 02");
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
  assert.equal(decode(doc("14 FF FF FF FF FF FF 1F 00")), 2 ** 53 - 1);
  assert.equal(decode(doc("14 01 00 00 00 00 00 E0 FF")), -(2 ** 53 - 1));
  assert.equal(decode(doc("14 00 00 00 00 00 00 20 00")), 2n ** 53n);
  assert.equal(decode(doc("14 00 00 00 00 00 00 E0 FF")), -(2n ** 53n));
  assert.equal(decode(doc("0A 00 80")), -(2 ** 15));
  assert.equal(decode(doc("0F 00 00 00 80")), -(2 ** 31));

  assert.equal(decode(doc("19 02 01")), 0.1);
  assert.ok(Object.is(decode(doc("19 01 00")), -0));
  // Writers never leave trailing zeros in the mantissa, but readers accept them: 150e-1.
  assert.equal(decode(doc("19 02 96 01")), 15);
  // A mantissa past 2^53 is parsed exactly rather than rounded first: 10000000596046448e-16.
  assert.equal(decode(doc("19 3E F0 E4 9F 9A A9 DE E1 11")), 1.0000000596046448);
  assert.ok(Number.isNaN(decode(doc("1E 00 00 00 00 00 00 F8 7F"))));

  const blob = doc("37 03 01 02 03");
  const view = decode<Uint8Array>(blob);
  assert.ok(view instanceof Uint8Array);
  assert.equal(view.buffer, blob.buffer, "a view into the input, not a copy");
  assert.deepEqual(decode(blob, { bytes: "array" }), [1, 2, 3]);
});

test("decimals read back exactly", () => {
  let state = 0x2545f491;
  const next = () => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return state >>> 0;
  };
  const varint = (value: number) => {
    const out: number[] = [];
    for (; value >= 0x80; value = Math.floor(value / 0x80)) {
      out.push((value % 0x80) | 0x80);
    }
    return [...out, value];
  };
  for (let i = 0; i < 10_000; i++) {
    // The fast path (mantissa up to 2^53, exponent up to 22) must agree with parsing the text.
    const mantissa = (next() % 2 ** 21) * 2 ** 32 + next();
    const exponent = (next() % 61) - 30;
    const zigzag = exponent < 0 ? -exponent * 2 - 1 : exponent * 2;
    const body = [ValueType.Decimal, ...varint(zigzag * 2), ...varint(mantissa)];
    const value = decode(new Uint8Array([0x53, 0x42, 0x01, 0x00, ...body]));
    assert.equal(value, Number(`${mantissa}e${exponent}`), `${mantissa}e${exponent}`);

    // Every number round-trips exactly in at most 9 bytes.
    const float = new Float64Array(new Uint32Array([next(), next()]).buffer)[0]!;
    if (Number.isNaN(float)) {
      continue;
    }
    const bytes = encode(float);
    assert.ok(bytes.length <= 4 + 9, `${float}`);
    assert.ok(Object.is(decode(bytes), Number.isSafeInteger(float) ? float + 0 : float), `${float}`);
  }
});
