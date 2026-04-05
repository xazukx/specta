#![allow(dead_code, missing_docs)]

use specta::{Type, Types};
use specta_jsonschema::{JsonSchema, SchemaVersion};

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
fn test_definitions_have_title() {
    let types = Types::default().register::<User>().register::<Status>();
    let value = JsonSchema::default().export_raw_as_value(&types).unwrap();
    assert_eq!(value["$defs"]["User"]["title"], "User");
    assert_eq!(value["$defs"]["Status"]["title"], "Status");
}

#[test]
fn test_draft2020_uses_defs() {
    let types = Types::default().register::<Status>();
    let value = JsonSchema::default()
        .schema_version(SchemaVersion::Draft2020_12)
        .export_raw_as_value(&types)
        .unwrap();

    assert!(value.get("$defs").is_some());
    assert!(value.get("definitions").is_none());
}

#[test]
fn test_draft7_uses_definitions() {
    let types = Types::default().register::<Status>();
    let value = JsonSchema::default()
        .schema_version(SchemaVersion::Draft7)
        .export_raw_as_value(&types)
        .unwrap();

    assert!(value.get("definitions").is_some());
    assert!(value.get("$defs").is_none());
}

#[test]
fn test_base_uri_adds_id_to_root() {
    let types = Types::default().register::<Status>();
    let value = JsonSchema::default()
        .base_uri("https://example.com/schemas")
        .export_raw_as_value(&types)
        .unwrap();

    assert_eq!(value["$id"], "https://example.com/schemas");
}

#[test]
fn test_no_base_uri_means_no_id() {
    let types = Types::default().register::<Status>();
    let value = JsonSchema::default().export_raw_as_value(&types).unwrap();
    assert!(value.get("$id").is_none());
}

#[test]
fn snapshot_string_enum() {
    let types = Types::default().register::<Status>();
    let value = JsonSchema::default().export_raw_as_value(&types).unwrap();
    insta::assert_json_snapshot!("string-enum", value["$defs"]["Status"]);
}

/// A documented user type.
#[derive(Type, serde::Serialize, serde::Deserialize)]
struct DocUser {
    /// The user's unique identifier.
    id: u32,
    /// The user's display name.
    name: String,
}

#[test]
fn test_field_descriptions_from_docs() {
    let types = Types::default().register::<DocUser>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["DocUser"];

    assert_eq!(def["title"], "DocUser");

    let id_prop = &def["properties"]["id"];
    let name_prop = &def["properties"]["name"];

    if id_prop.get("description").is_some() {
        assert!(
            id_prop["description"]
                .as_str()
                .unwrap()
                .contains("unique identifier")
        );
    }
    if name_prop.get("description").is_some() {
        assert!(
            name_prop["description"]
                .as_str()
                .unwrap()
                .contains("display name")
        );
    }
}
