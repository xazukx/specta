#![allow(dead_code, missing_docs)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::{Type, Types};
use specta_jsonschema::JsonSchema;

// --- Type definitions ---

#[derive(Type, Serialize, Deserialize)]
struct Base {
    id: u32,
    name: String,
}

#[derive(Type, Serialize, Deserialize)]
struct Extended {
    extra: bool,
    #[serde(flatten)]
    base: Base,
}

#[derive(Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CamelCaseStruct {
    first_name: String,
    last_name: String,
    is_active: bool,
}

#[derive(Type, Serialize, Deserialize)]
struct Metadata {
    created_by: String,
}

#[derive(Type, Serialize, Deserialize)]
struct Document {
    title: String,
    #[serde(flatten)]
    meta: Metadata,
}

#[derive(Type, Serialize, Deserialize)]
struct Additional {
    name: String,
    #[serde(flatten)]
    meta: HashMap<String, String>,
}

// --- Flatten tests ---

#[test]
fn test_flatten_produces_allof() {
    let types = Types::default().register::<Base>().register::<Extended>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["Extended"];

    assert!(def.get("allOf").is_some());
    let all_of = def["allOf"].as_array().unwrap();
    assert!(all_of.len() >= 2);
    assert!(all_of[0]["properties"]["extra"].is_object());
}

#[test]
fn test_flatten_with_ref() {
    let types = Types::default()
        .register::<Document>()
        .register::<Metadata>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["Document"];

    assert!(def.get("allOf").is_some());
    let all_of = def["allOf"].as_array().unwrap();
    assert!(all_of[0]["properties"]["title"].is_object());
    assert_eq!(all_of[1]["$ref"], "#/$defs/Metadata");
}

// --- additionalProperties ---

#[test]
fn test_flattened_struct_hashmap() {
    let types = Types::default().register::<Additional>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["Additional"];

    println!("{}", serde_json::to_string_pretty(&value).unwrap());
    let all_of = def.get("allOf");
    assert!(
        all_of.is_some(),
        "Expected allOf to exist: {}",
        serde_json::to_string_pretty(def).unwrap()
    );
    let all_of = all_of.unwrap();
    let additional_obj = all_of.get(1);
    assert!(
        additional_obj.is_some(),
        "Expected second element of allOf to exist: {}",
        serde_json::to_string_pretty(all_of).unwrap()
    );
    let additional_obj = additional_obj.unwrap();
    assert!(
        additional_obj.get("additionalProperties").is_some(),
        "Expected additionalProperties to exist: {}",
        serde_json::to_string_pretty(additional_obj).unwrap()
    );
}

// --- rename_all ---

#[test]
fn test_serde_rename_all_camel_case() {
    let types = Types::default().register::<CamelCaseStruct>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let props = value["$defs"]["CamelCaseStruct"]["properties"]
        .as_object()
        .unwrap();

    assert!(props.contains_key("firstName"));
    assert!(props.contains_key("lastName"));
    assert!(props.contains_key("isActive"));
    assert!(!props.contains_key("first_name"));
}

// --- Snapshots ---

#[test]
fn snapshot_flatten() {
    let types = Types::default().register::<Base>().register::<Extended>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("flatten", value["$defs"]["Extended"]);
}

#[test]
fn snapshot_flatten_with_ref() {
    let types = Types::default()
        .register::<Document>()
        .register::<Metadata>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("flatten-with-ref", value["$defs"]["Document"]);
}

#[test]
fn snapshot_rename_all() {
    let types = Types::default().register::<CamelCaseStruct>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("rename-all-camel", value["$defs"]["CamelCaseStruct"]);
}
