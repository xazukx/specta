#![allow(dead_code, missing_docs)]

use serde::{Deserialize, Serialize};
use specta::{Type, Types};
use specta_jsonschema::JsonSchema;

// --- Type definitions ---

#[derive(Type)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(Type, Serialize, Deserialize)]
#[serde(tag = "type")]
enum InternallyTagged {
    UnitVariant,
    Named { code: i32, message: String },
}

#[derive(Type, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum AdjacentlyTagged {
    UnitVariant,
    WithData(String),
    WithStruct { x: i32, y: i32 },
}

#[derive(Type, Serialize, Deserialize)]
#[serde(untagged)]
enum Untagged {
    Text(String),
    Number(i32),
}

#[derive(Type, Serialize, Deserialize)]
enum ExternallyTagged {
    Unit,
    Tuple(String),
    Struct { value: i32 },
}

#[derive(Type, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ScreamingEnum {
    FirstVariant,
    SecondVariant,
}

// --- Tests ---

#[test]
fn test_enum_values() {
    let types = Types::default().register::<Status>();
    let result = JsonSchema::default().export_raw(&types);
    assert!(result.is_ok());
    let schema = result.unwrap();
    assert!(schema.contains("\"Active\""));
    assert!(schema.contains("\"Inactive\""));
    assert!(schema.contains("\"Pending\""));
}

#[test]
fn test_string_enum_uses_enum_array() {
    let types = Types::default().register::<Status>();
    let value = JsonSchema::default().export_raw_as_value(&types).unwrap();

    let status_def = &value["$defs"]["Status"];
    assert!(
        status_def.get("enum").is_some(),
        "String-only enum should use {{\"enum\": [...]}} form, got: {}",
        serde_json::to_string_pretty(status_def).unwrap()
    );

    let enum_values = status_def["enum"].as_array().unwrap();
    assert_eq!(enum_values.len(), 3);
    assert!(enum_values.contains(&serde_json::json!("Active")));
}

#[test]
fn test_internal_tagging() {
    let types = Types::default().register::<InternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["InternallyTagged"];

    assert!(def.get("oneOf").is_some());

    let variants = def["oneOf"].as_array().unwrap();
    let unit = &variants[0];
    assert_eq!(unit["type"], "object");
    assert!(unit["properties"]["type"].is_object());

    let named = &variants[1];
    assert!(named["properties"]["type"].is_object());
    assert!(named["properties"]["code"].is_object());
    assert!(named["properties"]["message"].is_object());
}

#[test]
fn test_adjacent_tagging() {
    let types = Types::default().register::<AdjacentlyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["AdjacentlyTagged"];

    assert!(def.get("oneOf").is_some());

    let variants = def["oneOf"].as_array().unwrap();
    for v in variants {
        assert_eq!(v["type"], "object");
        assert!(v["properties"]["t"].is_object());
    }

    let with_data = &variants[1];
    assert!(with_data["properties"]["c"].is_object());
}

#[test]
fn test_untagged() {
    let types = Types::default().register::<Untagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["Untagged"];

    assert!(def.get("oneOf").is_some());

    let variants = def["oneOf"].as_array().unwrap();
    assert_eq!(variants.len(), 2);
    assert!(variants.iter().any(|v| v["type"] == "string"));
    assert!(variants.iter().any(|v| v["type"] == "integer"));
}

#[test]
fn test_external_tagging_with_serde() {
    let types = Types::default().register::<ExternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["ExternallyTagged"];

    assert!(def.get("oneOf").is_some());
    let variants = def["oneOf"].as_array().unwrap();
    assert_eq!(variants.len(), 3);

    let tuple_v = &variants[1];
    assert_eq!(tuple_v["type"], "object");
    assert!(tuple_v["properties"]["Tuple"].is_object());
}

#[test]
fn test_tag_literals_use_const() {
    let types = Types::default().register::<InternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["InternallyTagged"];

    let unit_tag = &def["oneOf"].as_array().unwrap()[0]["properties"]["type"];
    assert!(unit_tag.get("const").is_some());
    assert_eq!(unit_tag["const"], "UnitVariant");
}

#[test]
fn test_any_of_option() {
    let types = Types::default().register::<InternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default()
        .one_of(false)
        .export_as_value(&resolved)
        .unwrap();
    let def = &value["$defs"]["InternallyTagged"];

    assert!(def.get("anyOf").is_some());
    assert!(def.get("oneOf").is_none());
}

#[test]
fn test_default_uses_one_of() {
    let types = Types::default().register::<InternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["InternallyTagged"];

    assert!(def.get("oneOf").is_some());
    assert!(def.get("anyOf").is_none());
}

#[test]
fn test_serde_rename_all_enum() {
    let types = Types::default().register::<ScreamingEnum>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["ScreamingEnum"];
    let s = serde_json::to_string_pretty(def).unwrap();

    assert!(s.contains("FIRST_VARIANT"));
    assert!(s.contains("SECOND_VARIANT"));
}

// --- Snapshots ---

#[test]
fn snapshot_internal_tagging() {
    let types = Types::default().register::<InternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("internal-tagging", value);
}

#[test]
fn snapshot_adjacent_tagging() {
    let types = Types::default().register::<AdjacentlyTagged>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("adjacent-tagging", value);
}

#[test]
fn snapshot_untagged() {
    let types = Types::default().register::<Untagged>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("untagged", value);
}

#[test]
fn snapshot_external_tagging() {
    let types = Types::default().register::<ExternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("external-tagging", value);
}
