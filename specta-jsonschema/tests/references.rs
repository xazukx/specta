#![allow(dead_code, missing_docs)]

use serde::{Deserialize, Serialize};
use specta::{Type, Types};
use specta_jsonschema::{JsonSchema, Layout};

// --- Type definitions ---

#[derive(Type)]
enum Status {
    Active,
    Inactive,
}

#[derive(Type)]
struct WithRef {
    status: Status,
    name: String,
}

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

#[derive(Type, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum Event {
    Click { target: Address },
    Navigate { url: String },
}

#[derive(Type, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum AdjacentlyTagged {
    UnitVariant,
    WithData(String),
}

// --- $ref tests ---

#[test]
fn test_references() {
    let types = Types::default().register::<WithRef>().register::<Status>();
    let value = JsonSchema::default().export_raw_as_value(&types).unwrap();
    let def = &value["$defs"]["WithRef"];

    let status_prop = &def["properties"]["status"];
    assert!(status_prop.get("$ref").is_some());
    assert_eq!(status_prop["$ref"], "#/$defs/Status");
}

#[test]
fn test_struct_references_other_struct() {
    let types = Types::default().register::<Person>().register::<Address>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let person = &value["$defs"]["Person"];

    assert_eq!(person["properties"]["address"]["$ref"], "#/$defs/Address");
    assert_eq!(person["properties"]["name"]["type"], "string");
}

#[test]
fn test_enum_variant_references_other_struct() {
    let types = Types::default().register::<Event>().register::<Address>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let variants = value["$defs"]["Event"]["oneOf"].as_array().unwrap();
    let click = &variants[0];
    assert_eq!(click["properties"]["target"]["$ref"], "#/$defs/Address");
}

// --- Inline types ---

#[test]
fn test_string_fields_are_inlined() {
    let types = Types::default().register::<AdjacentlyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let with_data = &value["$defs"]["AdjacentlyTagged"]["oneOf"]
        .as_array()
        .unwrap()[1];
    assert_eq!(with_data["properties"]["c"]["type"], "string");
}

#[test]
fn test_inline_types_not_in_definitions() {
    let types = Types::default().register::<AdjacentlyTagged>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let defs = value["$defs"].as_object().unwrap();
    assert!(!defs.contains_key("String"));
}

// --- External $ref ---

#[test]
fn test_external_ref_override() {
    let js = JsonSchema::default().external_ref(
        "ExternalType",
        "https://cdn.example.com/schemas/ExternalType.json",
    );
    assert_eq!(
        js.build_ref("ExternalType"),
        "https://cdn.example.com/schemas/ExternalType.json"
    );
    assert_eq!(js.build_ref("User"), "#/$defs/User");
}

// --- Files layout references ---

#[test]
fn test_files_layout_uses_relative_refs() {
    let js = JsonSchema::default().layout(Layout::Files);
    assert_eq!(js.build_ref("Address"), "./Address.schema.json");
}

#[test]
fn test_files_layout_with_base_uri_uses_absolute_refs() {
    let js = JsonSchema::default()
        .layout(Layout::Files)
        .base_uri("https://example.com/schemas");
    assert_eq!(
        js.build_ref("Address"),
        "https://example.com/schemas/Address.schema.json"
    );
}

// --- Circular references ---

#[derive(Type, Serialize, Deserialize)]
struct TreeNode {
    label: String,
    children: Vec<TreeNode>,
}

#[test]
fn test_circular_reference() {
    let types = Types::default().register::<TreeNode>();
    let resolved = specta_serde::apply(types).unwrap();

    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    let def = &value["$defs"]["TreeNode"];

    assert_eq!(def["type"], "object");
    let items = &def["properties"]["children"]["items"];
    assert!(items.get("$ref").is_some());
    assert!(items["$ref"].as_str().unwrap().contains("TreeNode"));
}

// --- Snapshots ---

#[test]
fn snapshot_enum_with_struct_ref() {
    let types = Types::default().register::<Event>().register::<Address>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    println!("{}", serde_json::to_string_pretty(&value).unwrap());
    insta::assert_json_snapshot!("enum-with-struct-ref", value);
}

#[test]
fn snapshot_circular_ref() {
    let types = Types::default().register::<TreeNode>();
    let resolved = specta_serde::apply(types).unwrap();
    let value = JsonSchema::default().export_as_value(&resolved).unwrap();
    println!("{}", serde_json::to_string_pretty(&value).unwrap());
    insta::assert_json_snapshot!("circular-ref", value);
}

// --- Files layout: cross-file references ---

#[derive(Type, Serialize, Deserialize)]
struct Author {
    name: String,
    books: Vec<Book>,
}

#[derive(Type, Serialize, Deserialize)]
struct Book {
    title: String,
    author: Author,
}

#[test]
fn test_files_layout_cross_references() {
    let types = Types::default().register::<Author>().register::<Book>();
    let resolved = specta_serde::apply(types).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let js = JsonSchema::default().layout(Layout::Files);
    js.export_to(dir.path(), &resolved).unwrap();

    // Files are placed under a module subdirectory (the test crate name).
    // Find Author.schema.json wherever it was written.
    fn find_file(dir: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
        for entry in std::fs::read_dir(dir).ok()?.flatten() {
            let path = entry.path();
            if path.is_file() && path.file_name().is_some_and(|n| n == name) {
                return Some(path);
            }
            if path.is_dir() {
                if let Some(found) = find_file(&path, name) {
                    return Some(found);
                }
            }
        }
        None
    }

    let author_path =
        find_file(dir.path(), "Author.schema.json").expect("Author.schema.json should exist");
    let book_path =
        find_file(dir.path(), "Book.schema.json").expect("Book.schema.json should exist");

    let author_str = std::fs::read_to_string(&author_path).unwrap();
    let book_str = std::fs::read_to_string(&book_path).unwrap();
    let author: serde_json::Value = serde_json::from_str(&author_str).unwrap();
    let book: serde_json::Value = serde_json::from_str(&book_str).unwrap();

    println!(
        "Author:\n{}",
        serde_json::to_string_pretty(&author).unwrap()
    );
    println!("Book:\n{}", serde_json::to_string_pretty(&book).unwrap());

    // Author.books should reference Book via relative file path
    let books_items = &author["properties"]["books"]["items"];
    assert_eq!(
        books_items["$ref"], "./Book.schema.json",
        "Author.books items should $ref Book's file"
    );

    // Book.author should reference Author via relative file path
    let author_ref = &book["properties"]["author"];
    assert_eq!(
        author_ref["$ref"], "./Author.schema.json",
        "Book.author should $ref Author's file"
    );
}

#[test]
fn test_files_layout_cross_references_with_base_uri() {
    let types = Types::default().register::<Author>().register::<Book>();
    let resolved = specta_serde::apply(types).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let js = JsonSchema::default()
        .layout(Layout::Files)
        .base_uri("https://example.com/schemas");
    js.export_to(dir.path(), &resolved).unwrap();

    fn find_file2(dir: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
        for entry in std::fs::read_dir(dir).ok()?.flatten() {
            let path = entry.path();
            if path.is_file() && path.file_name().is_some_and(|n| n == name) {
                return Some(path);
            }
            if path.is_dir() {
                if let Some(found) = find_file2(&path, name) {
                    return Some(found);
                }
            }
        }
        None
    }

    let author_str = std::fs::read_to_string(
        find_file2(dir.path(), "Author.schema.json").expect("Author.schema.json"),
    )
    .unwrap();
    let author: serde_json::Value = serde_json::from_str(&author_str).unwrap();
    let book_str = std::fs::read_to_string(
        find_file2(dir.path(), "Book.schema.json").expect("Book.schema.json"),
    )
    .unwrap();
    let book: serde_json::Value = serde_json::from_str(&book_str).unwrap();
    println!("author: {author_str}");
    println!("book: {book_str}");

    // Each file should have its own $id
    assert_eq!(
        author["$id"],
        "https://example.com/schemas/Author.schema.json"
    );
    assert_eq!(book["$id"], "https://example.com/schemas/Book.schema.json");

    // Cross-references should use absolute URIs
    assert_eq!(
        author["properties"]["books"]["items"]["$ref"],
        "https://example.com/schemas/Book.schema.json"
    );
    assert_eq!(
        book["properties"]["author"]["$ref"],
        "https://example.com/schemas/Author.schema.json"
    );
}
