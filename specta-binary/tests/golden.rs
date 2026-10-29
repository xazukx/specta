//! Golden bytes: the exact wire output for the examples in `SPEC.md` and for the
//! boundaries between short and long forms, integer widths and special values.

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

fn no_table() -> Config {
    Config::new().key_table(false)
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
fn spec_example_with_key_table() {
    let bytes = ser(&ann());
    assert_eq!(
        bytes,
        from_hex(
            "53 42 01 01  03  04 6E 61 6D 65  05 65 6D 61 69 6C  03 61 67 65
             41 03  00 23 03 41 6E 6E  01 00  02 0A 1E 00 00 00"
        )
    );
    insta::assert_snapshot!(hex(&bytes));
}

#[test]
fn spec_example_without_key_table() {
    let bytes = to_vec_with(&ann(), &no_table()).expect("serialize");
    assert_eq!(
        bytes,
        from_hex(
            "53 42 01 00  41 03
             04 6E 61 6D 65  23 03 41 6E 6E
             05 65 6D 61 69 6C  00
             03 61 67 65  0A 1E 00 00 00"
        )
    );
    insta::assert_snapshot!(hex(&bytes));
}

#[test]
fn spec_example_without_objects() {
    // The key table is on, but there are no object keys, so no table is written.
    let bytes = ser(&vec![1, 2]);
    assert_eq!(
        bytes,
        from_hex("53 42 01 00  37 02  0A 01 00 00 00  0A 02 00 00 00")
    );
    insta::assert_snapshot!(hex(&bytes));
}

#[test]
fn short_long_boundaries() {
    insta::assert_snapshot!(cases([
        ("string 255", ser(&"a".repeat(255))),
        ("string 256", ser(&"a".repeat(256))),
        ("string 128 (2-byte varint)", ser(&"a".repeat(128))),
        ("string 16384 (3-byte varint)", ser(&"a".repeat(16384))),
        ("bytes 255", ser(serde_bytes::Bytes::new(&[7; 255]))),
        ("bytes 256", ser(serde_bytes::Bytes::new(&[7; 256]))),
        ("array 255", ser(&vec![(); 255])),
        ("array 256", ser(&vec![(); 256])),
        ("array 255 (unknown len)", ser(&UnknownLen(vec![(); 255]))),
        ("array 256 (unknown len)", ser(&UnknownLen(vec![(); 256]))),
        ("array declared 300, has 2", ser(&LyingLen(300, 2))),
        ("array declared 2, has 300", ser(&LyingLen(2, 300))),
        ("object 255", to_vec_with(&keys(255), &no_table()).expect("serialize")),
        ("object 256", to_vec_with(&keys(256), &no_table()).expect("serialize")),
        ("object 256 (key table)", ser(&keys(256))),
        ("object 255 (unknown len)", to_vec_with(&UnknownLenMap(keys(255)), &no_table()).expect("serialize")),
        ("object 256 (unknown len)", to_vec_with(&UnknownLenMap(keys(256)), &no_table()).expect("serialize")),
        ("key 200 bytes (inline)", to_vec_with(&BTreeMap::from([("k".repeat(200), 1)]), &no_table()).expect("serialize")),
    ]));
}

#[test]
fn numbers() {
    insta::assert_snapshot!(cases([
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
        ("f64 1.5", ser(&1.5f64)),
        ("f64 inf", ser(&f64::INFINITY)),
        ("f32 0.1", ser(&0.1f32)),
        ("f32 -0.0", ser(&-0.0f32)),
        ("true", ser(&true)),
        ("char é", ser(&'é')),
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

    let bytes = to_vec_with(
        &Bytes {
            plain: vec![1, 2, 3],
            compact: vec![1, 2, 3],
        },
        &no_table(),
    )
    .expect("serialize");
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
        ("empty string", ser(&"")),
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
fn externally_tagged_enum() {
    #[derive(Serialize)]
    enum E {
        Unit,
        Newtype(u8),
        Tuple(u8, u8),
        Struct { x: u8 },
    }

    insta::assert_snapshot!(cases([
        ("unit", to_vec_with(&E::Unit, &no_table()).expect("serialize")),
        ("newtype", to_vec_with(&E::Newtype(1), &no_table()).expect("serialize")),
        ("tuple", to_vec_with(&E::Tuple(1, 2), &no_table()).expect("serialize")),
        ("struct", to_vec_with(&E::Struct { x: 1 }, &no_table()).expect("serialize")),
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
