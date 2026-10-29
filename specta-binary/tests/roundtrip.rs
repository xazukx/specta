//! `from_slice(to_vec(x)) == x` across the serde data model, with the key and value tables
//! on and off.

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
};

use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use serde_json::json;
use specta_binary::{Config, from_slice, to_vec_with};

/// Every combination of key and value tables.
fn configs() -> [Config; 4] {
    [(true, true), (false, true), (true, false), (false, false)]
        .map(|(keys, values)| Config::new().key_table(keys).value_table(values))
}

/// Round-trip `value` with every table combination, and check the documents decode to the
/// same thing as each other and as `serde_json` (the "same shape as JSON" principle).
fn round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    round_trip_binary(value);
    // Through a string, not `serde_json::to_value`, which widens an `f32` to the exact `f64`
    // instead of writing its shortest decimal like `to_string` (and `Decimal`) do.
    let json: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(value).expect("serde_json serialize"))
            .expect("serde_json deserialize");
    for config in configs() {
        let bytes = to_vec_with(value, &config).expect("serialize");
        let decoded: serde_json::Value = from_slice(&bytes).expect("deserialize as Value");
        assert_eq!(decoded, json, "JSON shape with {config:?}");
    }
}

/// Like [`round_trip`], for types whose JSON shape can't be compared through
/// `serde_json::Value` (it can't hold raw bytes).
fn round_trip_binary<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    for config in configs() {
        let bytes = to_vec_with(value, &config).expect("serialize");
        let back: T = from_slice(&bytes).expect("deserialize");
        assert_eq!(&back, value, "round trip with {config:?}");
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: Option<String>,
    tags: Vec<String>,
    score: f64,
    active: bool,
}

fn user(id: u64) -> User {
    User {
        id,
        name: format!("user {id}"),
        email: id.is_multiple_of(2).then(|| format!("{id}@example.com")),
        tags: vec!["a".into(), "b".into()],
        score: id as f64 + 0.5,
        active: id.is_multiple_of(3),
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Tuple(u8, String, Option<i64>);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Unit;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Newtype(Vec<u16>);

#[test]
fn structs() {
    round_trip(&user(1));
    round_trip(&user(2));
    round_trip(&Tuple(1, "two".into(), Some(-3)));
    round_trip(&Tuple(1, "two".into(), None));
    round_trip(&Unit);
    round_trip(&Newtype(vec![1, 2, 3]));
    round_trip(&(1u8, -2i16, 3u32, "four".to_string(), 5.5f64, false));
}

#[test]
fn large_collections_use_long_forms() {
    // More than 127 distinct strings, so value ids take 2-byte varints.
    round_trip(&(0..300).map(user).collect::<Vec<_>>());
    round_trip(&"é".repeat(1000));
    round_trip(
        &(0..300)
            .map(|i| (format!("key {i}"), i))
            .collect::<BTreeMap<_, _>>(),
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum External {
    Unit,
    Newtype(String),
    Tuple(u8, Option<bool>),
    Struct { a: u8, b: Vec<External> },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Internal {
    Unit,
    Struct { a: u8, b: Option<String> },
    Newtype(User),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum Adjacent {
    Unit,
    Newtype(i64),
    Tuple(u8, u8),
    Struct { a: String },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum Untagged {
    Int(u8),
    Float(f64),
    Text(String),
    List(Vec<Untagged>),
    Object { x: i32, y: i32 },
    Null(()),
}

#[test]
fn enum_representations() {
    for value in [
        External::Unit,
        External::Newtype("x".into()),
        External::Tuple(1, Some(true)),
        External::Struct {
            a: 1,
            b: vec![External::Unit, External::Tuple(2, None)],
        },
    ] {
        round_trip(&value);
    }
    for value in [
        Internal::Unit,
        Internal::Struct { a: 1, b: None },
        Internal::Newtype(user(4)),
    ] {
        round_trip(&value);
    }
    for value in [
        Adjacent::Unit,
        Adjacent::Newtype(i64::MIN),
        Adjacent::Tuple(1, 2),
        Adjacent::Struct { a: "a".into() },
    ] {
        round_trip(&value);
    }
    for value in [
        Untagged::Int(5),
        Untagged::Float(2.5),
        Untagged::Text("hi".into()),
        Untagged::List(vec![Untagged::Int(1), Untagged::Text("x".into())]),
        Untagged::Object { x: -1, y: 2 },
        Untagged::Null(()),
    ] {
        round_trip(&value);
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Flattened {
    id: u32,
    #[serde(flatten)]
    inner: Inner,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Inner {
    name: String,
    count: u8,
}

// Kept apart from the catch-all map above: serde hands every key to a catch-all map
// when a sibling flattened struct itself contains a flattened field.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct NestedFlatten {
    id: u32,
    #[serde(flatten)]
    inner: Tagged,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Tagged {
    name: String,
    #[serde(flatten)]
    kind: Internal,
}

#[test]
fn flatten() {
    // Flatten serializes through `serialize_map(None)`, so this covers the unknown-length path.
    round_trip(&Flattened {
        id: 1,
        inner: Inner {
            name: "n".into(),
            count: 3,
        },
        extra: BTreeMap::from([
            ("x".into(), json!([1, 2])),
            ("y".into(), json!({ "z": null })),
        ]),
    });

    round_trip(&Flattened {
        id: 2,
        inner: Inner {
            name: "n".into(),
            count: 0,
        },
        extra: (0..300).map(|i| (format!("e{i}"), json!(i))).collect(),
    });

    for kind in [
        Internal::Unit,
        Internal::Struct {
            a: 3,
            b: Some("b".into()),
        },
    ] {
        round_trip(&NestedFlatten {
            id: 3,
            inner: Tagged {
                name: "n".into(),
                kind,
            },
        });
    }
}

fn double_option<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<Option<T>>, D::Error> {
    Option::deserialize(deserializer).map(Some)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Skipping {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    maybe: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    list: Vec<u8>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    patch: Option<Option<String>>,
}

#[test]
fn skip_serializing_if_and_double_option() {
    for patch in [None, Some(None), Some(Some("x".to_string()))] {
        round_trip(&Skipping {
            maybe: None,
            list: vec![],
            patch,
        });
    }
    round_trip(&Skipping {
        maybe: Some(1),
        list: vec![1],
        patch: None,
    });
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
enum Color {
    Red,
    Green,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
struct Id(u32);

#[test]
fn maps() {
    round_trip(&HashMap::from([
        (1u32, user(1)),
        (2, user(2)),
        (300, user(3)),
    ]));
    round_trip(&BTreeMap::from([
        ("a".to_string(), vec![1]),
        ("b".into(), vec![]),
    ]));
    round_trip(
        &BTreeMap::from([(-5i64, "neg"), (i64::MAX, "max")])
            .into_iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect::<BTreeMap<_, _>>(),
    );
    round_trip(&BTreeMap::from([(true, 1), (false, 0)]));
    round_trip(&BTreeMap::from([('x', 1), ('é', 2)]));
    round_trip(&BTreeMap::from([(Color::Red, 1), (Color::Green, 2)]));
    round_trip(&BTreeMap::from([(Id(7), 1)]));
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Tree {
    value: i32,
    children: Vec<Tree>,
    next: Option<Box<Tree>>,
}

#[test]
fn recursive_types() {
    let leaf = |value| Tree {
        value,
        children: vec![],
        next: None,
    };
    round_trip(&Tree {
        value: 1,
        children: vec![leaf(2), leaf(3)],
        next: Some(Box::new(Tree {
            value: 4,
            children: vec![leaf(5)],
            next: Some(Box::new(leaf(6))),
        })),
    });
}

#[test]
fn json_values() {
    round_trip(&json!({
        "null": null,
        "bool": true,
        "int": -12,
        "big": u64::MAX / 4,
        "float": 1.25,
        "string": "s",
        "array": [1, "two", [3.5], { "four": 4 }],
        "object": { "nested": { "deep": [] } },
    }));
}

#[test]
fn scalars() {
    round_trip(&'x');
    round_trip(&'🦀');
    round_trip(&i128::from(i64::MIN));
    round_trip(&u128::from(u64::MAX >> 1));
    round_trip(&(f64::MAX, f64::MIN_POSITIVE, -0.0f64));
    round_trip(&(1.5f32, f32::MAX, f32::MIN_POSITIVE));
    round_trip(&"\u{FEFF}starts with a BOM".to_string());
    round_trip(&());
    // Like JSON, `Some(())` is `null`, which reads back as `None`.
    round_trip(&Some(5u8));

    let nan: f64 =
        from_slice(&specta_binary::to_vec(&f64::NAN).expect("serialize")).expect("deserialize");
    assert!(nan.is_nan());
    let neg_zero: f64 =
        from_slice(&specta_binary::to_vec(&-0.0f64).expect("serialize")).expect("deserialize");
    assert!(neg_zero == 0.0 && neg_zero.is_sign_negative());
}

/// `Decimal` text must read back as exactly the same `f32` or `f64`, for every bit pattern.
#[test]
fn floats_round_trip_exactly() {
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..100_000 {
        let bits = next();
        let (f64_value, f32_value) = (f64::from_bits(bits), f32::from_bits(bits as u32));
        let bytes = specta_binary::to_vec(&(f64_value, f32_value)).expect("serialize");
        // A float never takes more than the 9 bytes of a `Float64`.
        assert!(bytes.len() <= 4 + 2 + 2 * 9, "{f64_value:e} {f32_value:e}");
        let (f64_back, f32_back): (f64, f32) = from_slice(&bytes).expect("deserialize");
        if !f64_value.is_nan() {
            assert_eq!(f64_back.to_bits(), f64_value.to_bits(), "{f64_value:e}");
        }
        if !f32_value.is_nan() {
            assert_eq!(f32_back.to_bits(), f32_value.to_bits(), "{f32_value:e}");
        }
    }
}

#[test]
fn integers_outside_i64_fail() {
    for result in [
        specta_binary::to_vec(&(i128::from(i64::MAX) + 1)),
        specta_binary::to_vec(&u128::MAX),
        specta_binary::to_vec(&u64::MAX),
    ] {
        assert!(result.is_err());
    }
}

#[test]
fn bytes() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Bytes {
        #[serde(with = "serde_bytes")]
        compact: Vec<u8>,
        plain: Vec<u8>,
    }
    round_trip_binary(&Bytes {
        compact: (0..=255).collect(),
        plain: vec![0, 255],
    });
    round_trip_binary(&serde_bytes::ByteBuf::from(vec![1; 1000]));
}

#[test]
fn borrows_from_the_input() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Borrowed<'a> {
        #[serde(borrow)]
        text: Cow<'a, str>,
        slice: &'a str,
        #[serde(with = "serde_bytes")]
        bytes: &'a [u8],
    }

    let value = Borrowed {
        text: Cow::Borrowed("cow"),
        slice: "slice",
        bytes: &[1, 2, 3],
    };
    for config in configs() {
        let bytes = to_vec_with(&value, &config).expect("serialize");
        let back: Borrowed<'_> = from_slice(&bytes).expect("deserialize");
        assert!(matches!(back.text, Cow::Borrowed(_)));
        assert_eq!(back, value);
    }
}

#[test]
fn unknown_fields_are_skipped() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Partial {
        name: String,
    }

    for config in configs() {
        let bytes = to_vec_with(&user(1), &config).expect("serialize");
        let partial: Partial = from_slice(&bytes).expect("deserialize");
        assert_eq!(partial.name, "user 1");
    }
}
