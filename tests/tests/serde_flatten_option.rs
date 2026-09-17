//! `#[serde(flatten)]` on an `Option<T>` contributes nothing when the value is `None` and merges
//! `T`'s fields into the parent object when it is `Some`. Exporting it as an intersection with
//! `T | null` was wrong in both directions: serde never serializes a bare `null` for the whole
//! value, and the legitimate `None` output (base fields only) does not satisfy `Base & T`. Each
//! export assertion below is paired with a `serde_json` test pinning down the real wire shapes.

use serde::{Deserialize, Serialize};
use specta::{Type, Types};
use specta_typescript::{Typescript, Zod};

#[derive(Type, Serialize, Deserialize)]
#[specta(collect = false)]
struct Inner1 {
    x: i32,
}

#[derive(Type, Serialize, Deserialize)]
#[specta(collect = false)]
struct Inner2 {
    y: i32,
}

#[derive(Type, Serialize, Deserialize)]
#[specta(collect = false)]
struct FlatOpt {
    a: i32,
    #[serde(flatten)]
    inner: Option<Inner1>,
}

#[derive(Type, Serialize, Deserialize)]
#[specta(collect = false)]
#[serde(tag = "t")]
enum InternalFlattenOpt {
    A {
        #[serde(flatten)]
        inner: Option<Inner1>,
    },
}

#[derive(Type, Serialize, Deserialize)]
#[specta(collect = false)]
struct FlatTwoOpt {
    a: i32,
    #[serde(flatten)]
    first: Option<Inner1>,
    #[serde(flatten)]
    second: Option<Inner2>,
}

#[derive(Type, Serialize, Deserialize, Default)]
#[specta(collect = false)]
#[serde(default)]
struct InnerDefault {
    z: i32,
}

#[derive(Type, Serialize, Deserialize)]
#[specta(collect = false)]
struct FlatDefault {
    a: i32,
    #[serde(flatten)]
    inner: InnerDefault,
}

#[derive(Type, Serialize, Deserialize)]
#[specta(collect = false)]
struct FlatNestedOpt {
    a: i32,
    #[serde(flatten)]
    inner: Option<Option<Inner1>>,
}

/// Collapses an export onto a single line, dropping the generated header, so the assertions read
/// like the code they pin down.
fn collapse(export: String) -> String {
    export
        .lines()
        .filter(|line| !line.starts_with("//"))
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Exports `T` as TypeScript with serde semantics applied.
fn export<T: Type>() -> String {
    let types = specta_serde::apply(Types::default().register::<T>())
        .expect("specta_serde::apply should succeed");

    collapse(
        Typescript::default()
            .export(&types)
            .expect("typescript export should succeed"),
    )
}

/// Exports `T` as Zod schemas with serde semantics applied, returning just `T`'s own schema so the
/// assertions aren't dominated by the import and `z.infer` aliases.
fn export_zod<T: Type>(name: &str) -> String {
    let types = specta_serde::apply(Types::default().register::<T>())
        .expect("specta_serde::apply should succeed");
    let export = collapse(
        Zod::default()
            .export(&types)
            .expect("zod export should succeed"),
    );

    let prefix = format!("export const {name}Schema = ");
    let start = export.find(&prefix).expect("schema should be exported") + prefix.len();
    let len = export[start..]
        .find(';')
        .expect("schema declaration should be terminated");

    export[start..start + len].to_string()
}

#[test]
fn flat_opt_serde_wire_shapes() {
    assert_eq!(
        serde_json::to_string(&FlatOpt { a: 1, inner: None }).unwrap(),
        r#"{"a":1}"#
    );
    assert_eq!(
        serde_json::to_string(&FlatOpt {
            a: 1,
            inner: Some(Inner1 { x: 2 })
        })
        .unwrap(),
        r#"{"a":1,"x":2}"#
    );
}

#[test]
fn flat_opt_exports_union_of_present_and_absent() {
    assert_eq!(
        export::<FlatOpt>(),
        r#"export type FlatOpt = { a: number, } & (Inner1) | { a: number, }; export type Inner1 = { x: number, };"#
    );
}

#[test]
fn internal_flatten_opt_serde_wire_shapes() {
    assert_eq!(
        serde_json::to_string(&InternalFlattenOpt::A { inner: None }).unwrap(),
        r#"{"t":"A"}"#
    );
    assert_eq!(
        serde_json::to_string(&InternalFlattenOpt::A {
            inner: Some(Inner1 { x: 1 })
        })
        .unwrap(),
        r#"{"t":"A","x":1}"#
    );
}

#[test]
fn internal_flatten_opt_exports_union_of_present_and_absent() {
    assert_eq!(
        export::<InternalFlattenOpt>(),
        r#"export type Inner1 = { x: number, }; export type InternalFlattenOpt = { t: "A" } & (Inner1) | { t: "A" };"#
    );
}

#[test]
fn flat_two_opt_serde_wire_shapes() {
    let shapes = [
        (None, None, r#"{"a":1}"#),
        (Some(Inner1 { x: 2 }), None, r#"{"a":1,"x":2}"#),
        (None, Some(Inner2 { y: 3 }), r#"{"a":1,"y":3}"#),
        (
            Some(Inner1 { x: 2 }),
            Some(Inner2 { y: 3 }),
            r#"{"a":1,"x":2,"y":3}"#,
        ),
    ];

    for (first, second, expected) in shapes {
        assert_eq!(
            serde_json::to_string(&FlatTwoOpt {
                a: 1,
                first,
                second
            })
            .unwrap(),
            expected
        );
    }
}

#[test]
fn flat_two_opt_exports_union_over_every_subset() {
    // The union is outermost so TypeScript's tighter-binding `&` never needs parentheses.
    assert_eq!(
        export::<FlatTwoOpt>(),
        r#"export type FlatTwoOpt = { a: number, } & (Inner1) & (Inner2) | { a: number, } & (Inner2) | { a: number, } & (Inner1) | { a: number, }; export type Inner1 = { x: number, }; export type Inner2 = { y: number, };"#
    );
}

#[test]
fn flat_default_serde_wire_shapes() {
    assert_eq!(
        serde_json::to_string(&FlatDefault {
            a: 1,
            inner: InnerDefault { z: 0 }
        })
        .unwrap(),
        r#"{"a":1,"z":0}"#
    );
}

#[test]
fn flat_default_exports_plain_intersection() {
    // A non-`Option` flattened field is always present, so it stays a mandatory intersection part.
    assert_eq!(
        export::<FlatDefault>(),
        r#"export type FlatDefault = { a: number, } & (InnerDefault); export type InnerDefault = { z?: number, };"#
    );
}

#[test]
fn flat_nested_opt_serde_wire_shapes() {
    assert_eq!(
        serde_json::to_string(&FlatNestedOpt { a: 1, inner: None }).unwrap(),
        r#"{"a":1}"#
    );
    assert_eq!(
        serde_json::to_string(&FlatNestedOpt {
            a: 1,
            inner: Some(Some(Inner1 { x: 2 }))
        })
        .unwrap(),
        r#"{"a":1,"x":2}"#
    );
}

#[test]
fn flat_nested_opt_exports_same_union_as_single_option() {
    assert_eq!(
        export::<FlatNestedOpt>(),
        r#"export type FlatNestedOpt = { a: number, } & (Inner1) | { a: number, }; export type Inner1 = { x: number, };"#
    );
}

#[test]
fn flat_opt_zod_unions_present_and_absent() {
    assert_eq!(
        export_zod::<FlatOpt>("FlatOpt"),
        r#"z.union([z.object({ a: z.int(), }).and(Inner1Schema), z.object({ a: z.int(), })])"#
    );
}

#[test]
fn internal_flatten_opt_zod_unions_present_and_absent() {
    assert_eq!(
        export_zod::<InternalFlattenOpt>("InternalFlattenOpt"),
        r#"z.union([z.object({ t: z.literal("A"), }).and(Inner1Schema), z.object({ t: z.literal("A"), })])"#
    );
}

#[test]
fn flat_two_opt_zod_unions_over_every_subset() {
    assert_eq!(
        export_zod::<FlatTwoOpt>("FlatTwoOpt"),
        r#"z.union([z.object({ a: z.int(), }).and(Inner1Schema.and(Inner2Schema)), z.object({ a: z.int(), }).and(Inner2Schema), z.object({ a: z.int(), }).and(Inner1Schema), z.object({ a: z.int(), })])"#
    );
}

#[test]
fn flat_default_zod_stays_a_plain_intersection() {
    assert_eq!(
        export_zod::<FlatDefault>("FlatDefault"),
        r#"z.object({ a: z.int(), }).and(InnerDefaultSchema)"#
    );
}

#[test]
fn flat_nested_opt_zod_matches_single_option() {
    assert_eq!(
        export_zod::<FlatNestedOpt>("FlatNestedOpt"),
        r#"z.union([z.object({ a: z.int(), }).and(Inner1Schema), z.object({ a: z.int(), })])"#
    );
}
