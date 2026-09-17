//! `specta-serde` decides whether an enum still needs rewriting to its serde wire representation by
//! tracking type identity (`split_types`/`generated_types` in `apply_phases`), never by inspecting
//! the shape of the variants it produced. That distinction matters because every shape the rewrite
//! emits is also a shape a user can author by hand, so a structural "is this already rewritten?"
//! check would be ambiguous in both directions.
//!
//! These tests pin down the shapes such a check would most likely misclassify: variants whose
//! payload is a bare string (what an external tag lowers to), a field named after its own variant,
//! and a partially `#[serde(untagged)]` enum under `apply_phases`, which rewrites each split type
//! before registering it.

use serde::{Deserialize, Serialize};
use specta::{Type, Types};
use specta_typescript::Typescript;

// An external tag lowers a newtype variant to a single `Primitive::str` field, which is exactly
// what `A(&'static str)` lowers to before any rewriting.
#[derive(Type, Serialize)]
#[specta(collect = false)]
enum StrNewtype {
    A(&'static str),
}

#[derive(Type, Serialize)]
#[specta(collect = false)]
enum StrField {
    A { s: &'static str },
}

// The rewrite names the wrapper field after the serialized variant name, which a user can also
// write directly via `#[serde(rename)]`.
#[derive(Type, Serialize)]
#[specta(collect = false)]
enum FieldNamedAfterVariant {
    Message {
        #[serde(rename = "Message")]
        text: String,
    },
}

#[derive(Type, Serialize, Deserialize)]
#[specta(collect = false)]
enum MixedUntagged {
    #[serde(alias = "a")]
    A(String),
    #[serde(untagged)]
    Other(u32),
}

/// Collapses an export onto a single line, dropping the generated header.
fn collapse(export: String) -> String {
    export
        .lines()
        .filter(|line| !line.starts_with("//"))
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

fn export<T: Type>() -> String {
    let types = specta_serde::apply(Types::default().register::<T>())
        .expect("specta_serde::apply should succeed");

    collapse(
        Typescript::default()
            .export(&types)
            .expect("typescript export should succeed"),
    )
}

fn export_phased<T: Type>() -> String {
    let types = specta_serde::apply_phases(Types::default().register::<T>())
        .expect("specta_serde::apply_phases should succeed");

    collapse(
        Typescript::default()
            .export(&types)
            .expect("typescript export should succeed"),
    )
}

#[test]
fn str_payload_variants_are_still_externally_tagged() {
    assert_eq!(
        serde_json::to_string(&StrNewtype::A("hi")).unwrap(),
        r#"{"A":"hi"}"#
    );
    assert_eq!(
        export::<StrNewtype>(),
        r#"export type StrNewtype = { A: string };"#
    );

    assert_eq!(
        serde_json::to_string(&StrField::A { s: "hi" }).unwrap(),
        r#"{"A":{"s":"hi"}}"#
    );
    assert_eq!(
        export::<StrField>(),
        r#"export type StrField = { A: { s: string, } };"#
    );
}

#[test]
fn field_named_after_its_variant_is_still_externally_tagged() {
    assert_eq!(
        serde_json::to_string(&FieldNamedAfterVariant::Message { text: "hi".into() }).unwrap(),
        r#"{"Message":{"Message":"hi"}}"#
    );
    assert_eq!(
        export::<FieldNamedAfterVariant>(),
        r#"export type FieldNamedAfterVariant = { Message: { Message: string, } };"#
    );
}

#[test]
fn partially_untagged_enum_is_tagged_exactly_once() {
    assert_eq!(
        serde_json::to_string(&MixedUntagged::A("x".into())).unwrap(),
        r#"{"A":"x"}"#
    );
    assert_eq!(
        serde_json::to_string(&MixedUntagged::Other(5)).unwrap(),
        "5"
    );

    // Not `{ A: { A: string } }` — the untagged variant must not leave the enum looking
    // unrewritten to the rewrite pass that runs after the split types are registered.
    assert_eq!(
        export::<MixedUntagged>(),
        r#"export type MixedUntagged = { A: string } | number;"#
    );
    assert_eq!(
        export_phased::<MixedUntagged>(),
        r#"export type MixedUntagged = { A: string } | number;"#
    );
}
