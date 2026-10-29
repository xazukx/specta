//! Malformed and hostile input must produce an error, never a panic, hang or unbounded allocation.

mod common;

use std::collections::BTreeMap;

use common::{doc, from_hex};
use serde::{Deserialize, Serialize, de::IgnoredAny};
use specta_binary::{Config, Error, ErrorKind, from_slice, from_slice_with, to_vec, to_vec_with};

#[derive(Debug, Serialize, Deserialize)]
struct Doc {
    users: Vec<User>,
    meta: BTreeMap<String, serde_json::Value>,
    #[serde(with = "serde_bytes")]
    blob: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    email: Option<String>,
    age: u8,
}

fn sample() -> Doc {
    Doc {
        users: (0..3)
            .map(|i| User {
                name: format!("ü{i}"),
                email: (i == 1).then(|| "a@b.c".into()),
                age: i,
            })
            .collect(),
        meta: BTreeMap::from([
            ("v".into(), serde_json::json!([1.5, -1, "x", null, true])),
            ("long".into(), serde_json::json!("y".repeat(300))),
        ]),
        blob: vec![9; 300],
    }
}

fn samples() -> Vec<Vec<u8>> {
    [Config::new(), Config::new().key_table(false)]
        .iter()
        .map(|config| to_vec_with(&sample(), config).expect("serialize"))
        .collect()
}

/// Decode `bytes` every way we can. Each must fail or succeed without panicking.
fn decode_all(bytes: &[u8]) -> [Result<(), Error>; 3] {
    [
        from_slice::<Doc>(bytes).map(drop),
        from_slice::<serde_json::Value>(bytes).map(drop),
        from_slice::<IgnoredAny>(bytes).map(drop),
    ]
}

/// Decode `bytes` by skipping and by visiting, which must fail the same way.
fn fail(bytes: &[u8]) -> Error {
    let skipped = from_slice::<IgnoredAny>(bytes).expect_err("should fail");
    let visited = from_slice::<serde_json::Value>(bytes).expect_err("should fail");
    assert_eq!(skipped.to_string(), visited.to_string(), "skipping and visiting agree");
    skipped
}

#[test]
fn samples_decode() {
    for bytes in samples() {
        let [typed, value, ignored] = decode_all(&bytes);
        typed.expect("valid document");
        ignored.expect("valid document");
        // `serde_json::Value` has no way to hold the raw bytes in `blob`.
        let err = value.expect_err("Value can't hold bytes");
        assert_eq!(err.path(), Some("$.blob"));
    }
}

#[test]
fn truncated_buffers() {
    for bytes in samples() {
        for len in 0..bytes.len() {
            for result in decode_all(&bytes[..len]) {
                assert!(result.is_err(), "prefix of {len} bytes decoded");
            }
        }
    }
}

#[test]
fn bad_header() {
    assert!(matches!(fail(&[]).kind(), ErrorKind::UnexpectedEof));
    assert!(matches!(fail(b"S").kind(), ErrorKind::UnexpectedEof));
    assert!(matches!(fail(&from_hex("53 43 01 00 00")).kind(), ErrorKind::InvalidMagic));
    assert!(matches!(fail(&from_hex("53 42 02 00 00")).kind(), ErrorKind::UnsupportedVersion(2)));
    assert!(matches!(fail(&from_hex("53 42 00 00 00")).kind(), ErrorKind::UnsupportedVersion(0)));
    assert!(matches!(fail(&from_hex("53 42 01 02 00")).kind(), ErrorKind::UnknownFlags(2)));
    assert!(matches!(fail(&from_hex("53 42 01 80 00")).kind(), ErrorKind::UnknownFlags(0x80)));
    assert!(matches!(fail(&from_hex("53 42 01")).kind(), ErrorKind::UnexpectedEof));
    // A header with no root value.
    assert!(matches!(fail(&from_hex("53 42 01 00")).kind(), ErrorKind::UnexpectedEof));

    let err = from_slice::<IgnoredAny>(&from_hex("53 42 01 02 00")).expect_err("bad flags");
    assert_eq!(err.path(), None);
    assert_eq!(err.offset(), Some(3));
}

#[test]
fn unknown_tags() {
    for tag in (0..=u8::MAX).filter(|tag| specta_binary::ValueType::try_from(*tag).is_err()) {
        assert!(matches!(fail(&doc(&format!("{tag:02X}"))).kind(), ErrorKind::UnknownTag(t) if *t == tag));
    }
    let err = from_slice::<IgnoredAny>(&doc("37 02 00 01")).expect_err("unknown tag");
    assert_eq!(err.to_string(), "at $[1]: unknown value type tag 0x01 (offset 7)");
}

#[test]
fn invalid_bool() {
    assert!(matches!(fail(&doc("05 02")).kind(), ErrorKind::InvalidBool(2)));
}

#[test]
fn huge_lengths_fail_before_allocating() {
    let huge = "FF FF FF FF 0F";
    for body in [
        format!("1E {huge}"), // string
        format!("28 {huge}"), // bytes
        format!("32 {huge}"), // array
        format!("3C {huge}"), // object
    ] {
        assert!(matches!(
            fail(&doc(&body)).kind(),
            ErrorKind::LengthOutOfBounds { len: u32::MAX, remaining: 0 }
        ));
    }
    // Key table count and key length.
    assert!(matches!(
        fail(&from_hex(&format!("53 42 01 01 {huge} 00"))).kind(),
        ErrorKind::LengthOutOfBounds { len: u32::MAX, .. }
    ));
    assert!(matches!(
        fail(&from_hex(&format!("53 42 01 01 01 {huge}"))).kind(),
        ErrorKind::LengthOutOfBounds { len: u32::MAX, .. }
    ));
    // Short lengths are checked too.
    assert!(matches!(fail(&doc("37 05 00")).kind(), ErrorKind::LengthOutOfBounds { len: 5, remaining: 1 }));
    // Typed targets that pre-allocate are covered by the same check.
    assert!(from_slice::<Vec<u64>>(&doc(&format!("32 {huge}"))).is_err());
    assert!(from_slice::<serde_bytes::ByteBuf>(&doc(&format!("32 {huge}"))).is_err());
}

#[test]
fn malformed_varints() {
    assert!(matches!(fail(&doc("1E 80 80 80 80 80 00")).kind(), ErrorKind::VarintOverflow));
    assert!(matches!(fail(&doc("1E FF FF FF FF 1F")).kind(), ErrorKind::VarintOverflow));
    assert!(matches!(fail(&doc("1E 80 80")).kind(), ErrorKind::UnexpectedEof));
}

#[test]
fn key_ids_out_of_range() {
    // One key in the table, but the object refers to id 1.
    let bytes = from_hex("53 42 01 01 01 01 61 41 01 01 00");
    assert!(matches!(fail(&bytes).kind(), ErrorKind::KeyIdOutOfRange { id: 1, count: 1 }));
}

#[test]
fn depth_limit() {
    let nested = |depth: usize| doc(&format!("{}00", "37 01 ".repeat(depth)));

    for result in decode_all(&nested(128)).into_iter().skip(1) {
        result.expect("128 levels are allowed");
    }
    assert!(matches!(fail(&nested(129)).kind(), ErrorKind::DepthLimitExceeded(128)));
    assert!(matches!(fail(&nested(100_000)).kind(), ErrorKind::DepthLimitExceeded(128)));

    let config = Config::new().max_depth(2);
    from_slice_with::<IgnoredAny>(&nested(2), &config).expect("within the limit");
    let err = from_slice_with::<serde_json::Value>(&nested(3), &config).expect_err("too deep");
    assert_eq!(err.to_string(), "at $[0][0]: maximum nesting depth of 2 exceeded (offset 8)");

    // Objects and enums count too.
    let objects = doc(&format!("{}00", "41 01 01 61 ".repeat(129)));
    assert!(matches!(fail(&objects).kind(), ErrorKind::DepthLimitExceeded(128)));
}

#[test]
fn invalid_utf8() {
    // In a value, an inline key and the key table.
    assert!(matches!(fail(&doc("23 02 C3 28")).kind(), ErrorKind::InvalidUtf8));
    assert!(matches!(fail(&doc("41 01 01 FF 00")).kind(), ErrorKind::InvalidUtf8));
    assert!(matches!(fail(&from_hex("53 42 01 01 01 01 FF 41 01 00 00")).kind(), ErrorKind::InvalidUtf8));
    // Overlong encodings and surrogates are invalid UTF-8 too.
    assert!(matches!(fail(&doc("23 02 C0 AF")).kind(), ErrorKind::InvalidUtf8));
    assert!(matches!(fail(&doc("23 03 ED A0 80")).kind(), ErrorKind::InvalidUtf8));
}

#[test]
fn trailing_bytes() {
    let mut bytes = to_vec(&1u8).expect("serialize");
    bytes.push(0);
    assert!(matches!(fail(&bytes).kind(), ErrorKind::TrailingBytes));
}

/// Flip bytes all over valid documents (a tiny deterministic fuzzer).
#[test]
fn mutations_never_panic() {
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for bytes in samples() {
        for _ in 0..5_000 {
            let mut mutated = bytes.clone();
            for _ in 0..=next() % 4 {
                let i = (next() as usize) % mutated.len();
                mutated[i] = next() as u8;
            }
            let _ = decode_all(&mutated);
        }
    }
}

#[test]
fn error_paths_and_offsets() {
    #[derive(Serialize)]
    struct BadRoot {
        users: Vec<BadUser>,
    }
    #[derive(Serialize)]
    struct BadUser {
        name: String,
        email: serde_json::Value,
        age: u8,
    }
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Root {
        users: Vec<User>,
    }

    let users = (0..5)
        .map(|i| BadUser {
            name: "n".into(),
            email: if i == 3 { serde_json::json!(7) } else { serde_json::json!(null) },
            age: 1,
        })
        .collect();
    let bytes = to_vec(&BadRoot { users }).expect("serialize");
    let mut messages = vec![from_slice::<Root>(&bytes).expect_err("bad email").to_string()];

    #[derive(Serialize)]
    struct Missing {
        users: Vec<BTreeMap<&'static str, &'static str>>,
    }
    let bytes = to_vec(&Missing {
        users: vec![BTreeMap::from([("name", "x")])],
    })
    .expect("serialize");
    messages.push(from_slice::<Root>(&bytes).expect_err("missing field").to_string());

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    enum E {
        A { x: u8 },
    }
    let bytes = to_vec(&BTreeMap::from([("A", BTreeMap::from([("x", "no")]))])).expect("serialize");
    messages.push(from_slice::<E>(&bytes).expect_err("bad variant field").to_string());
    let bytes = to_vec(&"B").expect("serialize");
    messages.push(from_slice::<E>(&bytes).expect_err("unknown variant").to_string());
    let bytes = to_vec(&BTreeMap::from([("weird key", 1.5)])).expect("serialize");
    messages.push(from_slice::<BTreeMap<String, String>>(&bytes).expect_err("not a string").to_string());

    insta::assert_snapshot!(messages.join("\n"));
}
