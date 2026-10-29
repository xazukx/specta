//! Golden bytes: the exact wire output for the examples in `SPEC.md` and for the
//! boundaries between short and long forms, integer widths, decimals and special values.

mod common;

use std::collections::BTreeMap;

use common::{from_hex, hex, summary};
use serde::{Serialize, Serializer as _};
use specta_binary::{Config, to_vec, to_vec_with};

#[derive(Serialize)]
struct User {
    name: String,
    email: Option<String>,
    age: u8,
}

fn ann() -> User {
    User {
        name: "Ann".into(),
        email: None,
        age: 30,
    }
}

/// No key table and no value table.
fn plain() -> Config {
    Config::new().key_table(false).value_table(false)
}

fn ser_plain<T: ?Sized + Serialize>(value: &T) -> Vec<u8> {
    to_vec_with(value, &plain()).expect("serialize")
}

fn ser<T: ?Sized + Serialize>(value: &T) -> Vec<u8> {
    to_vec(value).expect("serialize")
}

/// Render named cases as one `name: summary` line each.
fn cases(cases: impl IntoIterator<Item = (&'static str, Vec<u8>)>) -> String {
    cases
        .into_iter()
        .map(|(name, doc)| format!("{name:<28} {}\n", summary(&doc)))
        .collect()
}

/// A sequence whose length isn't known up front, like the ones `flatten` produces.
struct UnknownLen<T>(Vec<T>);

impl<T: Serialize> Serialize for UnknownLen<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter().filter(|_| true))
    }
}

/// A map whose length isn't known up front.
struct UnknownLenMap(BTreeMap<String, u8>);

impl Serialize for UnknownLenMap {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.0.iter().filter(|_| true))
    }
}

/// A sequence that declares the wrong length.
struct LyingLen(usize, usize);

impl Serialize for LyingLen {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.0))?;
        for i in 0..self.1 {
            seq.serialize_element(&(i as u8))?;
        }
        seq.end()
    }
}

fn keys(n: usize) -> BTreeMap<String, u8> {
    (0..n).map(|i| (format!("k{i:03}"), 0)).collect()
}

#[test]
fn spec_example_with_tables() {
    let bytes = ser(&ann());
    assert_eq!(
        bytes,
        from_hex(
            "53 42 01 03  03  04 6E 61 6D 65  05 65 6D 61 69 6C  03 61 67 65
             01  03 41 6E 6E
             4B 03  00 2D 00  01 00  02 0A 1E 00"
        )
    );
    insta::assert_snapshot!(hex(&bytes));
}

#[test]
fn spec_example_without_tables() {
    let bytes = ser_plain(&ann());
    assert_eq!(
        bytes,
        from_hex(
            "53 42 01 00  4B 03
             04 6E 61 6D 65  28 03 41 6E 6E
             05 65 6D 61 69 6C  00
             03 61 67 65  0A 1E 00"
        )
    );
    insta::assert_snapshot!(hex(&bytes));
}

#[test]
fn spec_example_without_strings() {
    // Both tables are on, but there are no object keys or strings, so no table is written.
    let bytes = ser(&vec![1, 2]);
    assert_eq!(bytes, from_hex("53 42 01 00  41 02  0A 01 00  0A 02 00"));
    insta::assert_snapshot!(hex(&bytes));
}

#[test]
fn spec_example_repeated_strings() {
    let bytes = ser(&["a", "b", "a"]);
    assert_eq!(
        bytes,
        from_hex("53 42 01 02  02 01 61 01 62  41 03  2D 00  2D 01  2D 00")
    );
    insta::assert_snapshot!(hex(&bytes));
}

#[test]
fn spec_example_decimals() {
    let bytes = ser(&(1.5, 19.99, -1e-7, 0.1f32, 0.1 + 0.2, f64::NAN));
    assert_eq!(
        bytes,
        from_hex(
            "53 42 01 00  41 06
             19 02 0F
             19 06 CF 0F
             19 1B 01
             19 02 01
             1E 34 33 33 33 33 33 D3 3F
             1E 00 00 00 00 00 00 F8 7F"
        )
    );
    insta::assert_snapshot!(hex(&bytes));
}

#[test]
fn short_long_boundaries() {
    insta::assert_snapshot!(cases([
        ("string 255", ser_plain(&"a".repeat(255))),
        ("string 256", ser_plain(&"a".repeat(256))),
        ("string 128 (2-byte varint)", ser_plain(&"a".repeat(128))),
        (
            "string 16384 (3-byte varint)",
            ser_plain(&"a".repeat(16384))
        ),
        ("string 256 (value table)", ser(&"a".repeat(256))),
        ("bytes 255", ser(serde_bytes::Bytes::new(&[7; 255]))),
        ("bytes 256", ser(serde_bytes::Bytes::new(&[7; 256]))),
        ("array 255", ser(&vec![(); 255])),
        ("array 256", ser(&vec![(); 256])),
        ("array 255 (unknown len)", ser(&UnknownLen(vec![(); 255]))),
        ("array 256 (unknown len)", ser(&UnknownLen(vec![(); 256]))),
        ("array declared 300, has 2", ser(&LyingLen(300, 2))),
        ("array declared 2, has 300", ser(&LyingLen(2, 300))),
        ("object 255", ser_plain(&keys(255))),
        ("object 256", ser_plain(&keys(256))),
        ("object 256 (key table)", ser(&keys(256))),
        (
            "object 255 (unknown len)",
            ser_plain(&UnknownLenMap(keys(255)))
        ),
        (
            "object 256 (unknown len)",
            ser_plain(&UnknownLenMap(keys(256)))
        ),
        (
            "key 200 bytes (inline)",
            ser_plain(&BTreeMap::from([("k".repeat(200), 1)]))
        ),
    ]));
}

#[test]
fn numbers() {
    insta::assert_snapshot!(cases([
        ("0", ser(&0)),
        ("i16::MIN", ser(&i16::MIN)),
        ("i16::MAX", ser(&i16::MAX)),
        ("i16::MAX + 1", ser(&(i32::from(i16::MAX) + 1))),
        ("i16::MIN - 1", ser(&(i32::from(i16::MIN) - 1))),
        ("u16::MAX", ser(&u16::MAX)),
        ("i32::MIN", ser(&i32::MIN)),
        ("i32::MAX", ser(&i32::MAX)),
        ("i32::MAX + 1", ser(&(i64::from(i32::MAX) + 1))),
        ("i32::MIN - 1", ser(&(i64::from(i32::MIN) - 1))),
        ("i64::MIN", ser(&i64::MIN)),
        ("i64::MAX as u64", ser(&(i64::MAX as u64))),
        ("u32::MAX", ser(&u32::MAX)),
        ("i128 -1", ser(&-1i128)),
        ("u128 7", ser(&7u128)),
        ("f64 NaN", ser(&f64::NAN)),
        ("f64 -0.0", ser(&-0.0f64)),
        ("f64 2.0", ser(&2.0f64)),
        ("f64 1.5", ser(&1.5f64)),
        ("f64 0.1", ser(&0.1f64)),
        ("f64 19.99", ser(&19.99f64)),
        ("f64 0.123456789012345", ser(&0.123_456_789_012_345_f64)),
        ("f64 0.1 + 0.2", ser(&(0.1f64 + 0.2))),
        ("f64 2^53", ser(&2f64.powi(53))),
        ("f64 1e21", ser(&1e21f64)),
        ("f64 1e-7", ser(&1e-7f64)),
        ("f64 MAX", ser(&f64::MAX)),
        ("f64 MIN", ser(&f64::MIN)),
        ("f64 5e-324", ser(&5e-324f64)),
        ("f64 inf", ser(&f64::INFINITY)),
        ("f64 -inf", ser(&f64::NEG_INFINITY)),
        ("f32 0.1", ser(&0.1f32)),
        ("f32 -0.0", ser(&-0.0f32)),
        ("f32 MAX", ser(&f32::MAX)),
        ("true", ser(&true)),
        ("char é", ser_plain(&'é')),
        ("char é (value table)", ser(&'é')),
        ("unit", ser(&())),
        ("none", ser(&None::<u8>)),
        ("some 5", ser(&Some(5u8))),
    ]));
}

#[test]
fn integers_that_dont_fit_in_i64() {
    let errors = [
        to_vec(&u64::MAX).map(drop),
        to_vec(&(i64::MAX as u64 + 1)).map(drop),
        to_vec(&(i128::from(i64::MIN) - 1)).map(drop),
        to_vec(&u128::MAX).map(drop),
        to_vec(&BTreeMap::from([("big", u64::MAX)])).map(drop),
    ];
    insta::assert_snapshot!(
        errors
            .into_iter()
            .map(|result| result.expect_err("should fail").to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn vec_u8_vs_serde_bytes() {
    #[derive(Serialize)]
    struct Bytes {
        plain: Vec<u8>,
        #[serde(with = "serde_bytes")]
        compact: Vec<u8>,
    }

    let bytes = ser_plain(&Bytes {
        plain: vec![1, 2, 3],
        compact: vec![1, 2, 3],
    });
    insta::assert_snapshot!(hex(&bytes));
}

#[test]
fn empty_containers() {
    #[derive(Serialize)]
    struct Empty {}

    insta::assert_snapshot!(cases([
        ("empty seq", ser(&Vec::<i32>::new())),
        ("empty map", ser(&BTreeMap::<String, i32>::new())),
        ("empty struct", ser(&Empty {})),
        ("empty string", ser_plain(&"")),
        ("empty string (value table)", ser(&"")),
        ("empty bytes", ser(serde_bytes::Bytes::new(&[]))),
    ]));
}

#[test]
fn key_ids_follow_first_use_order() {
    #[derive(Serialize)]
    struct Outer {
        a: Inner,
        c: u8,
        b: u8,
    }

    #[derive(Serialize)]
    struct Inner {
        b: u8,
        d: BTreeMap<u32, bool>,
    }

    // Depth-first: a, b (inside a), d, "1", c. The outer `b` reuses id 1.
    let bytes = ser(&Outer {
        a: Inner {
            b: 1,
            d: BTreeMap::from([(1, true)]),
        },
        c: 2,
        b: 3,
    });
    insta::assert_snapshot!(hex(&bytes));
}

#[test]
fn value_ids_follow_first_use_order() {
    #[derive(Serialize)]
    enum Role {
        Admin,
        Member,
    }

    #[derive(Serialize)]
    struct Member {
        name: &'static str,
        role: Role,
        initial: char,
    }

    // Values get ids in document order, separately from keys: "x", "Ann", "Member", "A",
    // "Admin", then "name", which is also in the key table but gets its own value id.
    let bytes = ser(&(
        "x",
        Member {
            name: "Ann",
            role: Role::Member,
            initial: 'A',
        },
        Member {
            name: "Ann",
            role: Role::Admin,
            initial: 'A',
        },
        "name",
        "x",
    ));
    insta::assert_snapshot!(hex(&bytes));
}

#[test]
fn externally_tagged_enum() {
    #[derive(Serialize)]
    enum E {
        Unit,
        Newtype(u8),
        Tuple(u8, u8),
        Struct { x: u8 },
    }

    insta::assert_snapshot!(cases([
        ("unit", ser_plain(&E::Unit)),
        ("newtype", ser_plain(&E::Newtype(1))),
        ("tuple", ser_plain(&E::Tuple(1, 2))),
        ("struct", ser_plain(&E::Struct { x: 1 })),
        ("unit (value table)", ser(&E::Unit)),
        ("struct (key table)", ser(&E::Struct { x: 1 })),
    ]));
}

#[test]
fn deterministic_output() {
    let value = (ann(), keys(300), "x".repeat(1000));
    assert_eq!(ser(&value), ser(&value));
}

#[test]
fn to_writer_matches_to_vec() {
    let mut out = Vec::new();
    specta_binary::to_writer(&mut out, &ann()).expect("serialize");
    assert_eq!(out, ser(&ann()));
}

#[test]
fn serializer_can_be_used_directly() {
    let mut serializer = specta_binary::Serializer::new(&Config::default());
    serializer.serialize_str("hi").expect("serialize");
    assert_eq!(serializer.into_bytes(), ser(&"hi"));
}
