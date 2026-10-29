// Decodes the documents written by `cargo run -p example-binary-ts` straight into the types
// `specta-typescript` generated. Run it with `node examples/binary-ts/main.ts` (Node 22.18+).
//
// `decode<T>` is an unchecked cast, like `JSON.parse(text) as T`.

import { readFileSync } from "node:fs";
import type { Attachment, Event, User } from "./out/bindings.ts";
import { decode, encode } from "./out/specta_binary.ts";

const read = (name: string) => new Uint8Array(readFileSync(new URL(`./out/${name}`, import.meta.url)));

const user = decode<User>(read("user.bin"));
console.log(`${user.name} <${user.email ?? "no email"}> is a ${user.role}`);

for (const event of decode<Event[]>(read("events.bin"))) {
  switch (event.type) {
    case "UserCreated":
      console.log(`created ${event.user.name} with ${event.user.followerCount} followers`);
      break;
    case "UserDeleted":
      console.log(`deleted user ${event.id}`);
      break;
    case "Ping":
      console.log("ping");
      break;
  }
}

// `serde_bytes` fields are typed `number[]`, so ask for arrays instead of `Uint8Array`.
const attachment = decode<Attachment>(read("attachment.bin"), { bytes: "array" });
console.log(attachment.name, attachment.data.map((byte) => byte.toString(16)).join(""));

// Encoding the same value gives the same bytes Rust wrote, ready to POST back.
const reencoded = encode(user);
console.log("byte-identical:", Buffer.from(reencoded).equals(read("user.bin")));

// Parse JSON before encoding it: the raw file `Buffer` would be encoded as a byte array.
console.log("--Big JSON--");
const big = JSON.parse(readFileSync(new URL("./big.json", import.meta.url), "utf8"));
const withTables = encode(big);
const inline = encode(big, { keyTable: false, valueTable: false });
console.log(`minified json: ${Buffer.byteLength(JSON.stringify(big))} B`);
console.log(`encoded: ${withTables.length} B, without the key and value tables: ${inline.length} B`);
