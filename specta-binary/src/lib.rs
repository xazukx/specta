//! Specta Binary (`SPB`): a compact, self-describing binary format for serde,
//! with a dependency-free TypeScript runtime that decodes it cheaply in the browser.
//!
//! It works the way `serde_json` and `JSON.parse` do: there is no schema and no code generation.
//! Any `Serialize`/`Deserialize` type works, and a decoded value has the same JavaScript shape as
//! `JSON.parse(serde_json::to_string(&value))`, so it can be cast straight to the types
//! `specta-typescript` generates. On top of the JSON data model the format adds typed numbers,
//! raw bytes, short lengths and an optional table of interned object keys.
//!
//! The wire format is specified in [`SPEC.md`](https://github.com/specta-rs/specta/blob/main/specta-binary/SPEC.md).
//!
//! # Usage
//!
//! ```
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct User {
//!     name: String,
//!     email: Option<String>,
//!     age: u8,
//! }
//!
//! let user = User { name: "Ann".into(), email: None, age: 30 };
//!
//! let bytes = specta_binary::to_vec(&user)?;
//! assert_eq!(specta_binary::from_slice::<User>(&bytes)?, user);
//! # Ok::<(), specta_binary::Error>(())
//! ```
//!
//! # The key table
//!
//! By default each document starts with a table of the object keys it uses, and every
//! object key is written as an index into that table. This makes documents with many
//! objects of the same shape (like a `Vec<User>`) much smaller. Turn it off per serialize
//! call with [`Config::key_table`]. Readers detect the table from the header, so
//! deserializing never needs to know which was used.
//!
//! ```
//! use specta_binary::Config;
//!
//! let users = vec![("name", "Ann"), ("name", "Bob")]
//!     .into_iter()
//!     .map(|(k, v)| std::collections::BTreeMap::from([(k, v)]))
//!     .collect::<Vec<_>>();
//!
//! let with_table = specta_binary::to_vec(&users)?;
//! let inline_keys = specta_binary::to_vec_with(&users, &Config::new().key_table(false))?;
//! assert_eq!(
//!     specta_binary::from_slice::<serde_json::Value>(&with_table)?,
//!     specta_binary::from_slice::<serde_json::Value>(&inline_keys)?,
//! );
//! # Ok::<(), specta_binary::Error>(())
//! ```
//!
//! # Bytes
//!
//! Like in JSON, serde serializes a plain `Vec<u8>` as an array of numbers. Use
//! [`serde_bytes`](https://docs.rs/serde_bytes) (`#[serde(with = "serde_bytes")]`) to get the
//! compact `ByteArray` form, which the TypeScript runtime decodes to a `Uint8Array`.
//! When deserializing, byte arrays and arrays of numbers are interchangeable.
//!
//! # TypeScript
//!
//! [`TYPESCRIPT_RUNTIME`] holds the source of the TypeScript runtime, which exports
//! `encode(value)` and `decode<T>(bytes)`. Write it into your frontend from a build script:
//!
//! ```no_run
//! std::fs::write("src/specta_binary.ts", specta_binary::TYPESCRIPT_RUNTIME)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Human-readable formats
//!
//! [`Serializer`] and [`Deserializer`] report `is_human_readable() == true`, so types like
//! `Uuid` and `DateTime` use the same string forms they use in JSON. This keeps the decoded
//! values in line with the types `specta-typescript` generates.
#![cfg_attr(docsrs, feature(doc_cfg))]

mod de;
mod error;
mod keys;
mod ser;
mod value_type;
mod varint;

use std::io;

use serde::{Deserialize, Serialize};

pub use de::Deserializer;
pub use error::{Error, ErrorKind};
pub use ser::{Compound, Serializer};
pub use value_type::{FLAG_HAS_KEY_TABLE, KNOWN_FLAGS, MAGIC, VERSION, ValueType};

/// The source of the TypeScript runtime (`specta_binary.ts`), so it can be written into a
/// frontend from a build script.
pub const TYPESCRIPT_RUNTIME: &str = include_str!("../runtime/specta_binary.ts");

/// Options for serializing and deserializing.
#[derive(Debug, Clone)]
pub struct Config {
    key_table: bool,
    max_depth: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            key_table: true,
            max_depth: 128,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether to intern object keys into a key table at the start of the document.
    /// Defaults to `true`. Only affects serializing: readers detect the table from the header.
    pub fn key_table(mut self, enabled: bool) -> Self {
        self.key_table = enabled;
        self
    }

    /// The maximum nesting depth of arrays and objects when deserializing. Defaults to `128`.
    pub fn max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }
}

/// Serialize `value` into a document, with the key table on.
pub fn to_vec<T: ?Sized + Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    to_vec_with(value, &Config::default())
}

/// Serialize `value` into a document using `config`.
pub fn to_vec_with<T: ?Sized + Serialize>(value: &T, config: &Config) -> Result<Vec<u8>, Error> {
    let mut serializer = Serializer::new(config);
    value.serialize(&mut serializer).map_err(Error::rooted)?;
    Ok(serializer.into_bytes())
}

/// Serialize `value` into `writer`. The document is buffered and written with a single `write_all`.
pub fn to_writer<W: io::Write, T: ?Sized + Serialize>(mut writer: W, value: &T) -> Result<(), Error> {
    writer.write_all(&to_vec(value)?)?;
    Ok(())
}

/// Deserialize a `T` from a document, with the default limits.
pub fn from_slice<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, Error> {
    from_slice_with(bytes, &Config::default())
}

/// Deserialize a `T` from a document. Only [`Config::max_depth`] applies.
pub fn from_slice_with<'de, T: Deserialize<'de>>(bytes: &'de [u8], config: &Config) -> Result<T, Error> {
    let mut deserializer = Deserializer::from_slice_with(bytes, config)?;
    let at = deserializer.offset();
    let value = T::deserialize(&mut deserializer).map_err(|err| err.or_offset(at).rooted())?;
    deserializer.end()?;
    Ok(value)
}
