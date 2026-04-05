#![allow(deprecated)]

use specta::{Constant, Constants, datatype::ConstantValue};
use specta_typescript::Typescript;

// #[specta_const] attribute macro on bare const items
#[specta::specta_const]
pub const BARE_MAX_RETRIES: u32 = 10;

#[specta::specta_const(name = "RENAMED_LIMIT")]
pub const SOME_LIMIT: u64 = 256;

#[specta::specta_const]
pub const BARE_MAGIC: [u8; 3] = [0xFF, 0xD8, 0xFF];

#[specta::specta_const(name = "ALPHABET", output = "string")]
// Should output `export const ALPHABET = "hello";`
pub const CHARS: &[u8] = b"hello";

#[specta::specta_const]
// Should output: `export const IS_ADMIN = true;`
pub const IS_ADMIN: bool = true;

mod tiny_text {
    pub struct TinyText(pub &'static str);
    pub struct TinyBytes(pub &'static [u8]);
}

pub struct ItemId(tiny_text::TinyText);
pub struct ItemBytes(tiny_text::TinyBytes);

#[specta::specta_const]
// Should output: `export const ITEM_ID = "item_id";`
pub const ITEM_ID: ItemId = ItemId(tiny_text::TinyText("item_id"));

#[specta::specta_const]
// Should output: `export const ITEM_BYTES = [105, 116, 101, 109, 95, 98, 121, 116, 101, 115];`
pub const ITEM_BYTES: ItemBytes = ItemBytes(tiny_text::TinyBytes(b"item_bytes"));

#[specta::specta_const]
// Should output: `export const ITEM_SCOPE = [105, 116, 101, 109, 95, 98, 121, 116, 101, 115];`
pub const ITEM_SCOPE: tiny_text::TinyBytes = tiny_text::TinyBytes(b"item_bytes");

// --- Tests ---

#[test]
fn test_specta_const_attribute_macro() {
    // #[specta_const] generates __specta_const_BARE_MAX_RETRIES implementing Constant
    assert_eq!(
        __specta_const_BARE_MAX_RETRIES::name().as_ref(),
        "BARE_MAX_RETRIES"
    );
    assert_eq!(
        __specta_const_BARE_MAX_RETRIES::value(),
        ConstantValue::UnsignedInteger(10)
    );
}

#[test]
fn test_specta_const_with_name_override() {
    assert_eq!(__specta_const_SOME_LIMIT::name().as_ref(), "RENAMED_LIMIT");
    assert_eq!(
        __specta_const_SOME_LIMIT::value(),
        ConstantValue::UnsignedInteger(256)
    );
}

#[test]
fn test_specta_const_with_output_format() {
    assert_eq!(
        __specta_const_BARE_MAGIC::value(),
        ConstantValue::Bytes(std::borrow::Cow::Owned(vec![0xFF, 0xD8, 0xFF]))
    );

    let types = specta::Types::default();
    let constants = Constants::default()
        .register::<__specta_const_BARE_MAX_RETRIES>()
        .register::<__specta_const_SOME_LIMIT>()
        .register::<__specta_const_BARE_MAGIC>();
    let resolved = specta::ResolvedTypes::from_resolved_types(types).with_constants(constants);

    let output = Typescript::default().export(&resolved).unwrap();
    println!("{output}");

    assert!(output.contains("export const BARE_MAX_RETRIES = 10 as const;"));
    assert!(output.contains("export const RENAMED_LIMIT = 256 as const;"));
    assert!(output.contains("export const BARE_MAGIC = [255, 216, 255] as const;"));
}

#[test]
fn test_specta_const_byte_string_as_string() {
    // b"hello" with output = "string" should produce a string ConstantValue
    assert_eq!(__specta_const_CHARS::name().as_ref(), "ALPHABET");
    assert_eq!(
        __specta_const_CHARS::value(),
        ConstantValue::String(std::borrow::Cow::Borrowed("hello"))
    );
}

#[test]
fn test_specta_const_bool() {
    assert_eq!(__specta_const_IS_ADMIN::name().as_ref(), "IS_ADMIN");
    assert_eq!(__specta_const_IS_ADMIN::value(), ConstantValue::Bool(true));
}

#[test]
fn test_specta_const_wrapper_string() {
    // ItemId(TinyText("item_id")) should extract the string literal
    assert_eq!(__specta_const_ITEM_ID::name().as_ref(), "ITEM_ID");
    assert_eq!(
        __specta_const_ITEM_ID::value(),
        ConstantValue::String(std::borrow::Cow::Borrowed("item_id"))
    );
}

#[test]
fn test_specta_const_wrapper_bytes() {
    // ItemBytes(TinyBytes(b"item_bytes")) should extract the byte string as bytes
    assert_eq!(__specta_const_ITEM_BYTES::name().as_ref(), "ITEM_BYTES");
    assert_eq!(
        __specta_const_ITEM_BYTES::value(),
        ConstantValue::Bytes(std::borrow::Cow::Owned(b"item_bytes".to_vec()))
    );
}

#[test]
fn test_specta_const_direct_wrapper_bytes() {
    // TinyBytes(b"item_bytes") should extract the byte string as bytes
    assert_eq!(__specta_const_ITEM_SCOPE::name().as_ref(), "ITEM_SCOPE");
    assert_eq!(
        __specta_const_ITEM_SCOPE::value(),
        ConstantValue::Bytes(std::borrow::Cow::Owned(b"item_bytes".to_vec()))
    );
}

#[test]
fn test_specta_const_typescript_output_all() {
    let types = specta::Types::default();
    let constants = Constants::default()
        .register::<__specta_const_BARE_MAX_RETRIES>()
        .register::<__specta_const_SOME_LIMIT>()
        .register::<__specta_const_BARE_MAGIC>()
        .register::<__specta_const_CHARS>()
        .register::<__specta_const_IS_ADMIN>()
        .register::<__specta_const_ITEM_ID>()
        .register::<__specta_const_ITEM_BYTES>()
        .register::<__specta_const_ITEM_SCOPE>();
    let resolved = specta::ResolvedTypes::from_resolved_types(types).with_constants(constants);

    let output = Typescript::default().export(&resolved).unwrap();
    println!("{output}");

    // Primitives
    assert!(output.contains("export const BARE_MAX_RETRIES = 10 as const;"));
    assert!(output.contains("export const RENAMED_LIMIT = 256 as const;"));
    assert!(output.contains("export const BARE_MAGIC = [255, 216, 255] as const;"));

    // Byte string as string
    assert!(output.contains("export const ALPHABET = \"hello\" as const;"));

    // Bool
    assert!(output.contains("export const IS_ADMIN = true as const;"));

    // Wrapper with string literal
    assert!(output.contains("export const ITEM_ID = \"item_id\" as const;"));

    // Wrapper with byte string → bytes array
    assert!(output.contains(
        "export const ITEM_BYTES = [105, 116, 101, 109, 95, 98, 121, 116, 101, 115] as const;"
    ));

    // Direct wrapper with byte string → bytes array
    assert!(output.contains(
        "export const ITEM_SCOPE = [105, 116, 101, 109, 95, 98, 121, 116, 101, 115] as const;"
    ));
}
