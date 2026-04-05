#![allow(dead_code, missing_docs)]

// --- Import const handling ---

#[test]
fn test_import_const_produces_enum() {
    let schema: schemars::Schema = serde_json::from_str(r#"{"const": "hello"}"#).unwrap();
    let dt = specta_jsonschema::import::from_schema(&schema).unwrap();

    match &dt {
        specta::datatype::DataType::Enum(e) => {
            assert_eq!(e.variants().len(), 1);
            assert_eq!(e.variants()[0].0.as_ref(), "hello");
        }
        other => panic!("Expected Enum for const, got: {:?}", other),
    }
}

#[test]
fn test_import_const_numeric() {
    let schema: schemars::Schema = serde_json::from_str(r#"{"const": 42}"#).unwrap();
    let dt = specta_jsonschema::import::from_schema(&schema).unwrap();

    match &dt {
        specta::datatype::DataType::Enum(e) => {
            assert_eq!(e.variants().len(), 1);
            // Numeric const gets stringified
            assert_eq!(e.variants()[0].0.as_ref(), "42");
        }
        other => panic!("Expected Enum for numeric const, got: {:?}", other),
    }
}

#[test]
fn test_import_string_enum() {
    let schema: schemars::Schema = serde_json::from_str(r#"{"enum": ["a", "b", "c"]}"#).unwrap();
    let dt = specta_jsonschema::import::from_schema(&schema).unwrap();

    match &dt {
        specta::datatype::DataType::Enum(e) => {
            assert_eq!(e.variants().len(), 3);
        }
        other => panic!("Expected Enum, got: {:?}", other),
    }
}

#[test]
fn test_import_nullable_pattern() {
    let schema: schemars::Schema =
        serde_json::from_str(r#"{"anyOf": [{"type": "string"}, {"type": "null"}]}"#).unwrap();
    let dt = specta_jsonschema::import::from_schema(&schema).unwrap();

    assert!(
        matches!(dt, specta::datatype::DataType::Nullable(_)),
        "anyOf [string, null] should produce Nullable"
    );
}

#[test]
fn test_import_object_with_properties() {
    let schema: schemars::Schema = serde_json::from_str(
        r#"{"type": "object", "properties": {"name": {"type": "string"}}, "required": ["name"]}"#,
    )
    .unwrap();
    let dt = specta_jsonschema::import::from_schema(&schema).unwrap();

    assert!(
        matches!(dt, specta::datatype::DataType::Struct(_)),
        "Object with properties should produce Struct"
    );
}
