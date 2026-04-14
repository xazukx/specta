#![allow(unused)]
use specta::{ResolvedTypes, Type, Types};
use specta_swift::Swift;

#[derive(Type)]
struct WithUuid {
    id: uuid::Uuid,
    name: String,
}

#[test]
fn test_uuid_support() {
    let types = Types::default().register::<WithUuid>();
    let resolved = ResolvedTypes::from_resolved_types(types);
    let swift = Swift::default();
    let output = swift.export(&resolved).unwrap();

    println!("UUID support test:\n{}", output);

    // UUID should be converted to String in Swift
    assert!(output.contains("let id: String"));
    assert!(output.contains("let name: String"));
}

#[test]
fn test_uuid_not_available() {
    println!("UUID feature not enabled - this is expected");
    // This test passes when UUID feature is not enabled
}

#[derive(Type)]
struct WithChrono {
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::NaiveDateTime,
    name: String,
}

#[test]
fn test_chrono_support() {
    let types = Types::default().register::<WithChrono>();
    let resolved = ResolvedTypes::from_resolved_types(types);
    let swift = Swift::default();
    let output = swift.export(&resolved).unwrap();

    println!("Chrono support test:\n{}", output);

    // Chrono types should be converted to String in Swift
    assert!(output.contains("let createdAt: String"));
    assert!(output.contains("let updatedAt: String"));
    assert!(output.contains("let name: String"));
}

#[test]
fn test_chrono_not_available() {
    println!("Chrono feature not enabled - this is expected");
    // This test passes when chrono feature is not enabled
}
