//! The TypeScript runtime's `ValueType` object must match the Rust enum.

use specta_binary::{TYPESCRIPT_RUNTIME, ValueType};

#[test]
fn typescript_value_types_match_rust() {
    let start = TYPESCRIPT_RUNTIME
        .find("export const ValueType = {")
        .expect("`ValueType` object in the runtime");
    let body = &TYPESCRIPT_RUNTIME[start..];
    let body =
        &body[body.find('{').expect("opening brace") + 1..body.find('}').expect("closing brace")];

    let typescript = body
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let (name, value) = entry.split_once(':').expect("`Name: value` entry");
            (
                name.trim().to_string(),
                value.trim().parse::<u8>().expect("numeric tag"),
            )
        })
        .collect::<Vec<_>>();

    let rust = ValueType::ALL
        .iter()
        .map(|tag| (tag.name().to_string(), *tag as u8))
        .collect::<Vec<_>>();

    assert_eq!(typescript, rust);
    for (_, byte) in &rust {
        assert_eq!(ValueType::try_from(*byte).map(|tag| tag as u8), Ok(*byte));
    }
}
