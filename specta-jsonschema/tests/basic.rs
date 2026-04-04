#![allow(dead_code, missing_docs)]

use serde::{Deserialize, Serialize};
use specta::{Type, Types};
use specta_jsonschema::{JsonSchema, SchemaVersion};

// --- Basic types ---

#[derive(Type)]
struct User {
    id: u32,
    name: String,
    email: Option<String>,
}

#[derive(Type)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[test]
fn test_basic_export() {
    let types = Types::default().register::<User>().register::<Status>();

    let result = JsonSchema::default().export_raw(&types);
    assert!(result.is_ok(), "Export should succeed: {:?}", result.err());

    let schema_str = result.unwrap();
    assert!(schema_str.contains("\"$schema\""));
    assert!(schema_str.contains("\"User\""));
    assert!(schema_str.contains("\"Status\""));
}

#[test]
fn test_schema_version() {
    let types = Types::default().register::<User>();

    let result = JsonSchema::default()
        .schema_version(SchemaVersion::Draft7)
        .export_raw(&types);

    assert!(result.is_ok());
    let schema = result.unwrap();
    assert!(schema.contains("http://json-schema.org/draft-07/schema#"));
}

#[test]
fn test_primitives() {
    #[derive(Type)]
    struct Primitives {
        string_field: String,
        int_field: i32,
        float_field: f64,
        bool_field: bool,
    }

    let types = Types::default().register::<Primitives>();
    let result = JsonSchema::default().export_raw(&types);

    assert!(result.is_ok());
    let schema = result.unwrap();
    assert!(schema.contains("\"type\": \"string\""));
    assert!(schema.contains("\"type\": \"integer\""));
    assert!(schema.contains("\"type\": \"number\""));
    assert!(schema.contains("\"type\": \"boolean\""));
}

#[test]
fn test_nullable() {
    let types = Types::default().register::<User>();
    let result = JsonSchema::default().export_raw(&types);

    assert!(result.is_ok());
    let schema = result.unwrap();
    assert!(schema.contains("anyOf") || schema.contains("null"));
}

#[test]
fn test_enum() {
    let types = Types::default().register::<Status>();
    let result = JsonSchema::default().export_raw(&types);

    assert!(result.is_ok());
    let schema = result.unwrap();
    assert!(schema.contains("\"Active\""));
    assert!(schema.contains("\"Inactive\""));
    assert!(schema.contains("\"Pending\""));
}

// --- String enum optimization ---

#[test]
fn test_string_enum_uses_enum_array() {
    let types = Types::default().register::<Status>();
    let value = JsonSchema::default().export_raw_as_value(&types).unwrap();

    let status_def = &value["definitions"]["Status"];
    assert!(
        status_def.get("enum").is_some(),
        "String-only enum should use {{\"enum\": [...]}} form, got: {}",
        serde_json::to_string_pretty(status_def).unwrap()
    );

    let enum_values = status_def["enum"].as_array().unwrap();
    assert_eq!(enum_values.len(), 3);
    assert!(enum_values.contains(&serde_json::json!("Active")));
    assert!(enum_values.contains(&serde_json::json!("Inactive")));
    assert!(enum_values.contains(&serde_json::json!("Pending")));
}

// --- Title and description on definitions ---

#[test]
fn test_definitions_have_title() {
    let types = Types::default().register::<User>().register::<Status>();
    let value = JsonSchema::default().export_raw_as_value(&types).unwrap();

    let user_def = &value["definitions"]["User"];
    assert_eq!(user_def["title"], "User");

    let status_def = &value["definitions"]["Status"];
    assert_eq!(status_def["title"], "Status");
}

// --- Enum tagging modes with specta-serde ---

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

#[test]
fn test_internal_tagging() {
    let types = Types::default().register::<InternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["definitions"]["InternallyTagged"];
    let schema_str = serde_json::to_string_pretty(def).unwrap();

    // Should have anyOf with variants that each include a "type" field
    assert!(
        def.get("anyOf").is_some(),
        "Internally tagged enum should use anyOf, got: {schema_str}"
    );

    let variants = def["anyOf"].as_array().unwrap();

    // Unit variant should be an object with just "type" property
    let unit = &variants[0];
    assert_eq!(unit["type"], "object");
    assert!(
        unit["properties"]["type"].is_object(),
        "Unit variant should have 'type' property, got: {}",
        serde_json::to_string_pretty(unit).unwrap()
    );

    // Named variant should be an object with "type", "code", "message" properties
    let named = &variants[1];
    assert_eq!(named["type"], "object");
    assert!(named["properties"]["type"].is_object());
    assert!(named["properties"]["code"].is_object());
    assert!(named["properties"]["message"].is_object());
}

#[test]
fn test_adjacent_tagging() {
    let types = Types::default().register::<AdjacentlyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["definitions"]["AdjacentlyTagged"];
    let schema_str = serde_json::to_string_pretty(def).unwrap();
    println!("{schema_str}");

    assert!(
        def.get("anyOf").is_some(),
        "Adjacently tagged enum should use anyOf, got: {schema_str}"
    );

    let variants = def["anyOf"].as_array().unwrap();

    // Each variant should have "t" as tag
    for v in variants {
        assert_eq!(v["type"], "object");
        assert!(
            v["properties"]["t"].is_object(),
            "Each variant should have 't' property, got: {}",
            serde_json::to_string_pretty(v).unwrap()
        );
    }

    // WithData variant should have "c" content field
    let with_data = &variants[1];
    assert!(
        with_data["properties"]["c"].is_object(),
        "WithData variant should have 'c' content property, got: {}",
        serde_json::to_string_pretty(with_data).unwrap()
    );
}

#[test]
fn test_untagged() {
    let types = Types::default().register::<Untagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["definitions"]["Untagged"];
    let schema_str = serde_json::to_string_pretty(def).unwrap();

    assert!(
        def.get("anyOf").is_some(),
        "Untagged enum should use anyOf, got: {schema_str}"
    );

    let variants = def["anyOf"].as_array().unwrap();
    assert_eq!(variants.len(), 2);

    // Untagged variants should be the raw types (no wrapping object).
    // String is an inline type so it should be inlined as {"type": "string"}.
    let has_string = variants.iter().any(|v| v["type"] == "string");
    let has_integer = variants.iter().any(|v| v["type"] == "integer");

    assert!(
        has_string,
        "Should have a string variant, got: {schema_str}"
    );
    assert!(
        has_integer,
        "Should have an integer variant, got: {schema_str}"
    );
}

#[test]
fn test_external_tagging_with_serde() {
    let types = Types::default().register::<ExternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["definitions"]["ExternallyTagged"];
    let schema_str = serde_json::to_string_pretty(def).unwrap();

    assert!(
        def.get("anyOf").is_some(),
        "Externally tagged enum should use anyOf, got: {schema_str}"
    );

    let variants = def["anyOf"].as_array().unwrap();
    assert_eq!(variants.len(), 3);

    // Unit variant: after serde transform, becomes a string literal enum with one variant
    let unit = &variants[0];
    let unit_str = serde_json::to_string(unit).unwrap();
    assert!(
        unit_str.contains("Unit"),
        "Unit variant should reference 'Unit', got: {unit_str}"
    );

    // Tuple variant: {"type": "object", "properties": {"Tuple": {...}}}
    let tuple_v = &variants[1];
    assert_eq!(tuple_v["type"], "object");
    assert!(
        tuple_v["properties"]["Tuple"].is_object(),
        "Tuple variant should have 'Tuple' property, got: {}",
        serde_json::to_string_pretty(tuple_v).unwrap()
    );

    // Struct variant: {"type": "object", "properties": {"Struct": {...}}}
    let struct_v = &variants[2];
    assert_eq!(struct_v["type"], "object");
    assert!(
        struct_v["properties"]["Struct"].is_object(),
        "Struct variant should have 'Struct' property, got: {}",
        serde_json::to_string_pretty(struct_v).unwrap()
    );
}

// --- Flattened struct fields ---

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

#[test]
fn test_flatten_produces_allof() {
    let types = Types::default().register::<Base>().register::<Extended>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["definitions"]["Extended"];
    let schema_str = serde_json::to_string_pretty(def).unwrap();

    assert!(
        def.get("allOf").is_some(),
        "Flattened struct should use allOf, got: {schema_str}"
    );

    let all_of = def["allOf"].as_array().unwrap();
    assert!(
        all_of.len() >= 2,
        "allOf should have at least 2 parts, got: {schema_str}"
    );

    // First part: the non-flattened fields
    let own_props = &all_of[0];
    assert!(own_props["properties"]["extra"].is_object());
}

// --- Schema version definitions key ---

#[test]
fn test_draft2020_uses_defs() {
    let types = Types::default().register::<Status>();
    let value = JsonSchema::default()
        .schema_version(SchemaVersion::Draft2020_12)
        .export_raw_as_value(&types)
        .unwrap();

    assert!(
        value.get("$defs").is_some(),
        "Draft 2020-12 should use $defs key"
    );
    assert!(
        value.get("definitions").is_none(),
        "Draft 2020-12 should not use definitions key"
    );
}

#[test]
fn test_draft7_uses_definitions() {
    let types = Types::default().register::<Status>();
    let value = JsonSchema::default()
        .schema_version(SchemaVersion::Draft7)
        .export_raw_as_value(&types)
        .unwrap();

    assert!(
        value.get("definitions").is_some(),
        "Draft 7 should use definitions key"
    );
    assert!(
        value.get("$defs").is_none(),
        "Draft 7 should not use $defs key"
    );
}

// --- Snapshot tests for stable output ---

#[test]
fn snapshot_string_enum() {
    let types = Types::default().register::<Status>();
    let value = JsonSchema::default().export_raw_as_value(&types).unwrap();
    insta::assert_json_snapshot!("string-enum", value["definitions"]["Status"]);
}

#[test]
fn snapshot_internal_tagging() {
    let types = Types::default().register::<InternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("internal-tagging", value["definitions"]["InternallyTagged"]);
}

#[test]
fn snapshot_adjacent_tagging() {
    let types = Types::default().register::<AdjacentlyTagged>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("adjacent-tagging", value["definitions"]["AdjacentlyTagged"]);
}

#[test]
fn snapshot_untagged() {
    let types = Types::default().register::<Untagged>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("untagged", value["definitions"]["Untagged"]);
}

#[test]
fn snapshot_external_tagging() {
    let types = Types::default().register::<ExternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("external-tagging", value["definitions"]["ExternallyTagged"]);
}

#[test]
fn snapshot_flatten() {
    let types = Types::default().register::<Base>().register::<Extended>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("flatten", value["definitions"]["Extended"]);
}

// --- References ---

#[derive(Type)]
struct WithRef {
    status: Status,
    name: String,
}

#[test]
fn test_references() {
    let types = Types::default().register::<WithRef>().register::<Status>();
    let value = JsonSchema::default().export_raw_as_value(&types).unwrap();
    let def = &value["definitions"]["WithRef"];

    // The status field should be a $ref
    let status_prop = &def["properties"]["status"];
    assert!(
        status_prop.get("$ref").is_some(),
        "Referenced type should produce $ref, got: {}",
        serde_json::to_string_pretty(status_prop).unwrap()
    );
    assert_eq!(status_prop["$ref"], "#/definitions/Status");
    println!("{}", serde_json::to_string_pretty(&value).unwrap());
}

// --- Inline types (String, Vec, etc.) should not produce $ref ---

#[test]
fn test_string_fields_are_inlined() {
    let types = Types::default().register::<AdjacentlyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["definitions"]["AdjacentlyTagged"];
    let schema_str = serde_json::to_string_pretty(def).unwrap();

    // WithData variant's "c" field should be inline {"type": "string"}, not $ref
    let with_data = &def["anyOf"].as_array().unwrap()[1];
    let c_field = &with_data["properties"]["c"];
    assert_eq!(
        c_field["type"], "string",
        "String content should be inlined as {{\"type\": \"string\"}}, not $ref. Got: {schema_str}"
    );
}

#[test]
fn test_inline_types_not_in_definitions() {
    let types = Types::default().register::<AdjacentlyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let defs = value["definitions"].as_object().unwrap();

    // String should NOT appear in definitions (it's an inline type)
    assert!(
        !defs.contains_key("String"),
        "Inline type 'String' should not appear in definitions. Keys: {:?}",
        defs.keys().collect::<Vec<_>>()
    );
}

// --- Tag literals use const, not enum array ---

#[test]
fn test_tag_literals_use_const() {
    let types = Types::default().register::<InternallyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["definitions"]["InternallyTagged"];

    let variants = def["anyOf"].as_array().unwrap();
    let unit_tag = &variants[0]["properties"]["type"];

    // Single-variant string enum used as tag should produce {"const": "..."}, not {"enum": [...]}
    assert!(
        unit_tag.get("const").is_some(),
        "Tag literal should use const, got: {}",
        serde_json::to_string_pretty(unit_tag).unwrap()
    );
    assert_eq!(unit_tag["const"], "UnitVariant");
}

// --- Struct and enum with references to other types ---

#[derive(Type, Serialize, Deserialize)]
struct Address {
    street: String,
    city: String,
}

#[derive(Type, Serialize, Deserialize)]
struct Person {
    name: String,
    address: Address,
}

#[test]
fn test_struct_references_other_struct() {
    let types = Types::default().register::<Person>().register::<Address>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let person = &value["definitions"]["Person"];

    // address field should be a $ref to Address
    let addr_prop = &person["properties"]["address"];
    assert_eq!(
        addr_prop["$ref"], "#/definitions/Address",
        "Struct field referencing another struct should use $ref"
    );

    // name field should be inlined as string
    let name_prop = &person["properties"]["name"];
    assert_eq!(
        name_prop["type"], "string",
        "String field should be inlined"
    );
}

#[derive(Type, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum Event {
    Click { target: Address },
    Navigate { url: String },
}

#[test]
fn test_enum_variant_references_other_struct() {
    let types = Types::default().register::<Event>().register::<Address>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["definitions"]["Event"];
    let schema_str = serde_json::to_string_pretty(def).unwrap();

    let variants = def["anyOf"].as_array().unwrap();

    // Click variant should have "target" that references Address
    let click = &variants[0];
    let target_prop = &click["properties"]["target"];
    assert_eq!(
        target_prop["$ref"], "#/definitions/Address",
        "Enum variant field referencing another struct should use $ref. Got: {schema_str}"
    );
}

#[test]
fn snapshot_enum_with_struct_ref() {
    let types = Types::default().register::<Event>().register::<Address>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("enum-with-struct-ref", value["definitions"]["Event"]);
}

// --- serde rename_all ---

#[derive(Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CamelCaseStruct {
    first_name: String,
    last_name: String,
    is_active: bool,
}

#[test]
fn test_serde_rename_all_camel_case() {
    let types = Types::default().register::<CamelCaseStruct>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["definitions"]["CamelCaseStruct"];
    let props = def["properties"].as_object().unwrap();

    assert!(
        props.contains_key("firstName"),
        "Should have camelCase 'firstName', got keys: {:?}",
        props.keys().collect::<Vec<_>>()
    );
    assert!(props.contains_key("lastName"));
    assert!(props.contains_key("isActive"));
    assert!(!props.contains_key("first_name"));
}

#[derive(Type, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ScreamingEnum {
    FirstVariant,
    SecondVariant,
}

#[test]
fn test_serde_rename_all_enum() {
    let types = Types::default().register::<ScreamingEnum>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["definitions"]["ScreamingEnum"];
    let schema_str = serde_json::to_string_pretty(def).unwrap();

    assert!(
        schema_str.contains("FIRST_VARIANT"),
        "Should have SCREAMING_SNAKE_CASE variant names. Got: {schema_str}"
    );
    assert!(schema_str.contains("SECOND_VARIANT"));
}

#[test]
fn snapshot_rename_all() {
    let types = Types::default().register::<CamelCaseStruct>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("rename-all-camel", value["definitions"]["CamelCaseStruct"]);
}

// --- Flatten with enum variant ---

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

#[test]
fn test_flatten_with_inline_type() {
    let types = Types::default()
        .register::<Document>()
        .register::<Metadata>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["definitions"]["Document"];
    let schema_str = serde_json::to_string_pretty(def).unwrap();

    assert!(
        def.get("allOf").is_some(),
        "Flattened struct should use allOf. Got: {schema_str}"
    );

    // First part should have "title"
    let all_of = def["allOf"].as_array().unwrap();
    let own_props = &all_of[0];
    assert!(own_props["properties"]["title"].is_object());

    // Second part should reference Metadata
    let flat_ref = &all_of[1];
    assert_eq!(flat_ref["$ref"], "#/definitions/Metadata");
}

#[test]
fn snapshot_flatten_with_ref() {
    let types = Types::default()
        .register::<Document>()
        .register::<Metadata>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    insta::assert_json_snapshot!("flatten-with-ref", value["definitions"]["Document"]);
}
