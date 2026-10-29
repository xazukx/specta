// Cross-language conformance: Rust writes fixtures with `to_vec` and `serde_json`, this checks
// that `decode` matches `JSON.parse`, that re-encoding reproduces the bytes, and writes
// `fixtures/ts/*.bin` for Rust to decode. Run through `cargo test -p specta-tests binary`.

import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { type DecodeOptions, decode, encode } from "../../specta-binary/runtime/specta_binary.ts";

const fixtures = fileURLToPath(new URL("./fixtures/", import.meta.url));

interface Fixture {
  name: string;
  reencode: boolean;
  decode: DecodeOptions;
}

if (!existsSync(`${fixtures}manifest.json`)) {
  throw new Error("No fixtures found. Run `cargo test -p specta-tests binary` to write them.");
}
const manifest: Fixture[] = JSON.parse(readFileSync(`${fixtures}manifest.json`, "utf8"));

// Every combination of key table and value table, the same list as `VARIANTS` in `binary.rs`.
const variants = [
  { file: "bin", options: { keyTable: true, valueTable: true } },
  { file: "keys.bin", options: { keyTable: true, valueTable: false } },
  { file: "values.bin", options: { keyTable: false, valueTable: true } },
  { file: "plain.bin", options: { keyTable: false, valueTable: false } },
] as const;

const hex = (bytes: Uint8Array) => Buffer.from(bytes).toString("hex");

for (const fixture of manifest) {
  test(`decode ${fixture.name} matches JSON.parse`, () => {
    const json: unknown = JSON.parse(readFileSync(`${fixtures}${fixture.name}.json`, "utf8"));
    for (const { file } of variants) {
      // `readFileSync` returns a `Buffer` that may sit at an offset in a shared pool.
      const bin = readFileSync(`${fixtures}${fixture.name}.${file}`);
      assert.deepStrictEqual(decode(bin, { bytes: "array", ...fixture.decode }), json, file);
    }
  });

  if (fixture.reencode) {
    test(`re-encode ${fixture.name} is byte-identical`, () => {
      for (const { file, options } of variants) {
        const bin = new Uint8Array(readFileSync(`${fixtures}${fixture.name}.${file}`));
        assert.equal(hex(encode(decode(bin), options)), hex(bin), file);
      }
    });
  }
}

const users = Array.from({ length: 300 }, (_, i) => ({
  name: `user ${i}`,
  email: `${i}@example.com`,
  age: i % 100,
}));

// Values the TS encoder writes for Rust to decode, in `tests/tests/binary.rs`.
const tsFixtures: Record<string, unknown> = {
  user: { name: "Ann", email: null, age: 30 },
  user_without_email: { name: "Bob", email: undefined, age: 41 },
  users,
  bytes: { compact: new Uint8Array([1, 2, 3]), plain: [4, 5, 6] },
  bytes_swapped: { compact: [1, 2, 3], plain: new Uint8Array([4, 5, 6]) },
  numbers: {
    small: 1,
    negative: -5,
    big: 2 ** 40,
    bigint: 2n ** 60n,
    float: 1.5,
    intAsFloat: 2,
    floatAsF32: 0.1,
  },
  special_floats: [NaN, Infinity, -Infinity],
  external_enum: ["Unit", { Newtype: "x" }, { Tuple: [1, null] }, { Struct: { a: 2, b: [] } }],
  internal_enum: { type: "Struct", a: 1, b: "x" },
  integer_keys: { 1: "a", 20: "b" },
  long: {
    text: "x".repeat(300),
    list: Array.from({ length: 300 }, () => 1),
    map: Object.fromEntries(Array.from({ length: 300 }, (_, i) => [`k${i}`, i])),
  },
  unicode: "héllo 🦀 ﻿",
  date: new Date(0),
  repeated_strings: [["a", "b", "a"], ["Red", "Red"], { Red: "Red" }],
  // Unsafe integers like 2^53 are written as `Decimal` too.
  decimals: [0.1, 0.1, 1e21, 5e-324, -1.5e-7, 2 ** 53],
  json_value: { a: [1, "two", null, true, { b: 2.5 }], big: 2n ** 60n, nested: { c: [] } },
};

test("encode fixtures for Rust", () => {
  mkdirSync(`${fixtures}ts`, { recursive: true });
  for (const [name, value] of Object.entries(tsFixtures)) {
    for (const { file, options } of variants) {
      writeFileSync(`${fixtures}ts/${name}.${file}`, encode(value, options));
    }
  }
});

test("TS round trip", () => {
  for (const [name, value] of Object.entries(tsFixtures)) {
    if (name === "date" || name === "user_without_email") {
      continue;
    }
    for (const { options } of variants) {
      assert.deepStrictEqual(decode(encode(value, options)), value, name);
    }
  }
});
