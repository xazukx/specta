//! With no schema, readers accept any encoding of a value that a JS encoder could produce for it.

mod common;

use std::collections::HashMap;

use common::doc;
use serde::Deserialize;
use serde_bytes::ByteBuf;
use specta_binary::{from_slice, to_vec};

fn ser<T: serde::Serialize>(value: &T) -> Vec<u8> {
    to_vec(value).expect("serialize")
}

#[test]
fn integers_feed_float_targets() {
    assert_eq!(from_slice::<f64>(&ser(&5i32)).expect("i32 -> f64"), 5.0);
    assert_eq!(from_slice::<f32>(&ser(&5i32)).expect("i32 -> f32"), 5.0);
    assert_eq!(from_slice::<f64>(&ser(&(1i64 << 40))).expect("i64 -> f64"), (1u64 << 40) as f64);
    assert_eq!(from_slice::<f32>(&ser(&(1i64 << 40))).expect("i64 -> f32"), (1u64 << 40) as f32);
}

#[test]
fn floats_convert_between_widths() {
    assert_eq!(from_slice::<f32>(&ser(&0.1f64)).expect("f64 -> f32"), 0.1f32);
    assert_eq!(from_slice::<f64>(&ser(&0.5f32)).expect("f32 -> f64"), 0.5);
}

#[test]
fn integer_widths_are_range_checked() {
    assert_eq!(from_slice::<u8>(&ser(&255i64)).expect("fits"), 255);
    assert_eq!(from_slice::<u64>(&ser(&i64::MAX)).expect("fits"), i64::MAX as u64);
    assert_eq!(from_slice::<i8>(&doc("0F 80 FF FF FF FF FF FF FF")).expect("Int64 -> i8"), -128);

    let err = from_slice::<u8>(&ser(&256)).expect_err("out of range");
    assert_eq!(err.to_string(), "at $: integer 256 is out of range for u8 (offset 4)");
    let err = from_slice::<u32>(&ser(&-1)).expect_err("negative");
    assert_eq!(err.to_string(), "at $: integer -1 is out of range for u32 (offset 4)");
    let err = from_slice::<i32>(&ser(&1.0f64)).expect_err("floats don't feed integers");
    assert_eq!(err.to_string(), "at $: expected i32, found Float64 (offset 4)");
}

#[test]
fn byte_arrays_and_arrays_are_interchangeable() {
    let compact = ser(&ByteBuf::from(vec![1, 2, 255]));
    let plain = ser(&vec![1u8, 2, 255]);

    assert_eq!(from_slice::<Vec<u8>>(&compact).expect("bytes -> Vec<u8>"), [1, 2, 255]);
    assert_eq!(from_slice::<[u8; 3]>(&compact).expect("bytes -> [u8; 3]"), [1, 2, 255]);
    assert_eq!(from_slice::<Vec<u32>>(&compact).expect("bytes -> Vec<u32>"), [1, 2, 255]);
    assert_eq!(from_slice::<ByteBuf>(&plain).expect("array -> bytes"), [1, 2, 255].as_slice());

    let err = from_slice::<ByteBuf>(&ser(&vec![1, 256])).expect_err("not a byte");
    assert_eq!(err.to_string(), "at $[1]: integer 256 is out of range for u8 (offset 11)");
    let err = from_slice::<[u8; 2]>(&compact).expect_err("too long");
    assert!(err.to_string().contains("1 unexpected trailing elements"), "{err}");
}

#[test]
fn long_tags_with_short_lengths() {
    assert_eq!(from_slice::<String>(&doc("1E 03 61 62 63")).expect("StringUtf8"), "abc");
    assert_eq!(
        from_slice::<ByteBuf>(&doc("28 02 01 02")).expect("ByteArray"),
        [1, 2].as_slice()
    );
    assert_eq!(
        from_slice::<Vec<bool>>(&doc("32 02 05 01 05 00")).expect("Array"),
        [true, false]
    );
    assert_eq!(
        from_slice::<HashMap<String, bool>>(&doc("3C 01 01 61 05 01")).expect("Object"),
        HashMap::from([("a".to_string(), true)])
    );
    // A non-minimal varint is still a valid length.
    assert_eq!(from_slice::<String>(&doc("1E 81 00 61")).expect("padded varint"), "a");
}

#[test]
fn string_keys_feed_other_key_types() {
    let map = HashMap::from([("1".to_string(), 'a'), ("20".into(), 'b')]);
    assert_eq!(
        from_slice::<HashMap<u8, char>>(&ser(&map)).expect("parsed keys"),
        HashMap::from([(1, 'a'), (20, 'b')])
    );
    let bools = HashMap::from([("true".to_string(), 1)]);
    assert_eq!(
        from_slice::<HashMap<bool, u8>>(&ser(&bools)).expect("bool keys"),
        HashMap::from([(true, 1)])
    );

    let bad = HashMap::from([("one".to_string(), 1)]);
    let err = from_slice::<HashMap<u8, u8>>(&ser(&bad)).expect_err("not a number");
    assert!(err.to_string().contains("invalid type: string \"one\", expected u8"), "{err}");
}

#[test]
fn structs_from_arrays() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Point {
        x: i32,
        y: i32,
    }
    assert_eq!(
        from_slice::<Point>(&ser(&(1, 2))).expect("struct from array"),
        Point { x: 1, y: 2 }
    );
}

#[test]
fn enum_forms() {
    #[derive(Debug, PartialEq, Deserialize)]
    enum E {
        A,
        B(u8),
    }
    // A unit variant may also arrive as `{ "A": null }`.
    assert_eq!(from_slice::<E>(&doc("41 01 01 41 00")).expect("unit as object"), E::A);
    assert_eq!(from_slice::<E>(&doc("23 01 41")).expect("unit as string"), E::A);
    assert_eq!(from_slice::<E>(&doc("41 01 01 42 0A 07 00 00 00")).expect("newtype"), E::B(7));

    let err = from_slice::<E>(&doc("41 02 01 41 00 01 42 00")).expect_err("two entries");
    assert_eq!(
        err.to_string(),
        "at $: expected an enum object with exactly 1 entry, found 2 entries (offset 4)"
    );
}
