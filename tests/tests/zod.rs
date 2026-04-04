use std::{iter, path::Path};

use serde::{Deserialize, Serialize};
use specta::{
    ResolvedTypes, Type, Types,
    datatype::{DataType, NamedDataType, Primitive, Reference},
};
use specta_typescript::Typescript;
use specta_zod::{BigIntExportBehavior, Layout, Zod, ZodVersion, primitives};
use tempfile::TempDir;

macro_rules! for_bigint_types {
    (T -> $s:expr) => {{
        for_bigint_types!(usize, isize, i64, u64, i128, u128; $s);
    }};
    ($($i:ty),+; $s:expr) => {{
        $({
            type T = $i;
            $s(stringify!($i));
        })*
    }};
}

#[derive(Type)]
#[specta(collect = false)]
struct StructWithBigInt {
    a: i128,
}

#[derive(Type)]
#[specta(collect = false)]
struct StructWithStructWithBigInt {
    #[specta(inline)]
    abc: StructWithBigInt,
}

#[derive(Type)]
#[specta(collect = false)]
struct StructWithOptionWithStructWithBigInt {
    #[specta(inline)]
    optional_field: Option<StructWithBigInt>,
}

#[derive(Type)]
#[specta(collect = false)]
enum EnumWithInlineStructWithBigInt {
    #[specta(inline)]
    B { a: i128 },
}

#[derive(Type)]
struct Recursive {
    child: Option<Box<Recursive>>,
}

#[derive(Type)]
struct Testing {
    a: testing::Testing,
}

#[derive(Type)]
struct Another {
    bruh: String,
}

#[derive(Type)]
#[specta(collect = false)]
struct EmptyStruct {}

#[derive(Type)]
#[specta(collect = false)]
enum EmptyNamedVariant {
    A {},
}

#[derive(Type, Serialize, Deserialize)]
#[specta(collect = false)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum SerdeTaggedEnum {
    Unit,
    StringValue(String),
}

#[derive(Type, Serialize)]
#[specta(collect = false)]
#[serde(tag = "type")]
enum InvalidInternallyTaggedEnum {
    A(String),
}

mod testing {
    use super::*;

    #[derive(Type)]
    pub struct Testing {
        b: testing2::Testing,
    }

    pub mod testing2 {
        use super::*;

        #[derive(Type)]
        pub struct Testing {
            c: String,
        }
    }
}

fn inline_for<T: Type>(zod: &Zod) -> Result<String, specta_zod::Error> {
    let mut types = Types::default();
    let dt = T::definition(&mut types);
    primitives::inline(zod, &ResolvedTypes::from_resolved_types(types), &dt)
}

fn temp_root() -> std::path::PathBuf {
    let temp_root = Path::new(env!("CARGO_MANIFEST_DIR")).join(".temp");
    std::fs::create_dir_all(&temp_root).unwrap();
    temp_root
}

#[test]
fn zod_export_smoke() {
    #[derive(Type)]
    struct Inner {
        value: String,
    }

    #[derive(Type)]
    struct Demo {
        inner: Inner,
        count: i32,
        maybe: Option<String>,
    }

    let types = Types::default().register::<Demo>();
    let resolved = ResolvedTypes::from_resolved_types(types);

    let out = Zod::default()
        .bigint(BigIntExportBehavior::Number)
        .export(&resolved)
        .unwrap();

    assert!(out.contains("import { z } from \"zod\";"));
    assert!(out.contains("export const DemoSchema = z.object({\n\tinner: InnerSchema,\n\tcount: z.number(),\n\tmaybe: z.string().nullable(),\n});"));
    assert!(out.contains("export type Demo = z.infer<typeof DemoSchema>;"));
    assert!(out.contains("export const InnerSchema = z.object({\n\tvalue: z.string(),\n});"));
    assert!(out.contains("export type Inner = z.infer<typeof InnerSchema>;"));
}

#[test]
fn zod_primitives_smoke() {
    let (types, dts) = crate::types();
    let resolved = ResolvedTypes::from_resolved_types(types);

    let zod = Zod::default().bigint(BigIntExportBehavior::Number);

    for (_, ty) in &dts {
        let rendered = primitives::inline(&zod, &resolved, ty).unwrap();
        assert!(!rendered.is_empty());
    }

    let ndt = dts
        .iter()
        .find_map(|(_, ty)| match ty {
            DataType::Reference(Reference::Named(r)) => r.get(resolved.as_types()),
            _ => None,
        })
        .unwrap();

    let rendered = primitives::export(&zod, &resolved, iter::once(ndt), "").unwrap();
    assert!(rendered.contains("Schema"));
}

#[test]
fn zod_bigint_export_behaviors() {
    for_bigint_types!(T -> |_| {
        assert!(inline_for::<T>(&Zod::default()).is_err());
        assert!(inline_for::<T>(&Zod::default().bigint(BigIntExportBehavior::Fail)).is_err());

        assert_eq!(
            inline_for::<T>(&Zod::default().bigint(BigIntExportBehavior::String)).unwrap(),
            "z.string()"
        );
        assert_eq!(
            inline_for::<T>(&Zod::default().bigint(BigIntExportBehavior::Number)).unwrap(),
            "z.number()"
        );
        assert_eq!(
            inline_for::<T>(&Zod::default().bigint(BigIntExportBehavior::BigInt)).unwrap(),
            "z.bigint()"
        );
    });
}

#[test]
fn zod_bigint_errors_propagate_from_nested_types() {
    for err in [
        export_for::<StructWithBigInt>(),
        export_for::<StructWithStructWithBigInt>(),
        export_for::<StructWithOptionWithStructWithBigInt>(),
        export_for::<EnumWithInlineStructWithBigInt>(),
    ] {
        let err = err.expect_err("bigint export should be rejected by default");
        assert!(
            err.to_string().contains("forbids exporting BigInt types"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn zod_layout_duplicate_typenames() {
    let types = Types::default().register::<Testing>().register::<Another>();
    let resolved = ResolvedTypes::from_resolved_types(types.clone());

    let err = Zod::default().export(&resolved).unwrap_err();
    assert!(err.to_string().contains("Detected multiple types"));

    let module_prefixed = Zod::default()
        .layout(Layout::ModulePrefixedName)
        .export(&resolved)
        .unwrap();
    assert!(module_prefixed.contains("TestingSchema"));
    assert!(module_prefixed.contains("testing2"));
}

#[test]
fn zod_layout_files_export_to() {
    let types = Types::default().register::<Testing>().register::<Another>();
    let resolved = ResolvedTypes::from_resolved_types(types.clone());

    let temp = temp_dir();
    let path = temp.path().join("zod-layout-files");

    Zod::default()
        .layout(Layout::Files)
        .export_to(&path, &resolved)
        .unwrap();

    let output = crate::fs_to_string(Path::new(&path)).unwrap();
    assert!(output.contains(".ts"));
    assert!(output.contains("import { z } from \"zod\";"));
}

#[test]
fn zod_uses_serde_transformed_resolved_types() {
    let types = Types::default().register::<SerdeTaggedEnum>();

    let raw = ResolvedTypes::from_resolved_types(types.clone());
    let serde = specta_serde::apply(types).unwrap();

    let raw_out = Zod::default().export(&raw).unwrap();
    let serde_out = Zod::default().export(&serde).unwrap();

    assert_ne!(raw_out, serde_out);
    assert!(serde_out.contains("type: z.literal(\"unit\")"));
    assert!(serde_out.contains("type: z.literal(\"string_value\")"));
    assert!(serde_out.contains("data: z.string()"));
}

#[test]
fn zod_rejects_invalid_serde_shapes_via_transformation() {
    let types = Types::default().register::<InvalidInternallyTaggedEnum>();
    let err = specta_serde::apply(types).unwrap_err();

    assert!(err.to_string().contains("Invalid internally tagged enum"));
}

#[test]
fn zod_empty_named_shapes_are_strict() {
    let empty_struct = export_for::<EmptyStruct>().unwrap();
    assert!(empty_struct.contains("export const EmptyStructSchema = z.object({}).strict();"));

    let empty_variant = export_for::<EmptyNamedVariant>().unwrap();
    assert!(
        empty_variant.contains("export const EmptyNamedVariantSchema = z.object({}).strict();")
    );
}

#[test]
fn zod_layout_files_preserves_unrelated_typescript_files() {
    let types = Types::default().register::<Testing>().register::<Another>();
    let resolved = ResolvedTypes::from_resolved_types(types);

    let temp = TempDir::new_in(temp_root()).unwrap();
    let path = temp.path().join("zod-layout-files-preserve");
    std::fs::create_dir_all(&path).unwrap();

    let keep_path = path.join("keep.ts");
    std::fs::write(&keep_path, "export const keep = true;\n").unwrap();

    Zod::default()
        .layout(Layout::Files)
        .export_to(&path, &resolved)
        .unwrap();

    assert!(keep_path.exists());
    assert!(
        std::fs::read_to_string(&keep_path)
            .unwrap()
            .contains("export const keep = true;")
    );
}

#[test]
fn typescript_layout_files_preserves_unrelated_typescript_files() {
    let types = Types::default().register::<Testing>().register::<Another>();
    let resolved = ResolvedTypes::from_resolved_types(types);

    let temp = TempDir::new_in(temp_root()).unwrap();
    let path = temp.path().join("typescript-layout-files-preserve");
    std::fs::create_dir_all(&path).unwrap();

    let keep_path = path.join("keep.ts");
    std::fs::write(&keep_path, "export const keep = true;\n").unwrap();

    Typescript::default()
        .layout(specta_typescript::Layout::Files)
        .export_to(&path, &resolved)
        .unwrap();

    assert!(keep_path.exists());
    assert!(
        std::fs::read_to_string(&keep_path)
            .unwrap()
            .contains("export const keep = true;")
    );
}

#[test]
fn zod_recursive_types_use_lazy() {
    let types = Types::default().register::<Recursive>();
    let resolved = ResolvedTypes::from_resolved_types(types);

    let out = Zod::default().export(&resolved).unwrap();
    assert!(out.contains("export const RecursiveSchema = z.object({\n\tchild: z.lazy(() => RecursiveSchema).nullable(),\n});"));
}

#[test]
fn zod_reserved_type_name_errors() {
    let mut types = Types::default();
    NamedDataType::new("class", Vec::new(), DataType::Primitive(Primitive::i8))
        .register(&mut types);
    let resolved = ResolvedTypes::from_resolved_types(types);

    let err = Zod::default().export(&resolved).unwrap_err();
    assert!(err.to_string().contains("reserved keyword"));
}

#[test]
fn zod_layout_files_errors_on_export() {
    let types = Types::default();
    let resolved = ResolvedTypes::from_resolved_types(types);

    let err = Zod::default()
        .layout(Layout::Files)
        .export(&resolved)
        .unwrap_err();
    assert!(err.to_string().contains("Unable to export layout Files"));
}

fn temp_dir() -> TempDir {
    TempDir::new_in(temp_root()).unwrap()
}

fn export_for<T: Type>() -> Result<String, specta_zod::Error> {
    let types = Types::default().register::<T>();
    Zod::default().export(&ResolvedTypes::from_resolved_types(types))
}

fn export_for_v4<T: Type>() -> Result<String, specta_zod::Error> {
    let types = Types::default().register::<T>();
    Zod::default()
        .zod_version(ZodVersion::V4)
        .export(&ResolvedTypes::from_resolved_types(types))
}

fn inline_for_v4<T: Type>() -> Result<String, specta_zod::Error> {
    let zod = Zod::default().zod_version(ZodVersion::V4);
    inline_for_with::<T>(&zod)
}

fn inline_for_with<T: Type>(zod: &Zod) -> Result<String, specta_zod::Error> {
    let mut types = Types::default();
    let dt = T::definition(&mut types);
    primitives::inline(zod, &ResolvedTypes::from_resolved_types(types), &dt)
}

// --- Zod v4 tests ---

#[test]
fn zod_v4_empty_named_shapes_use_strict_object() {
    let empty_struct = export_for_v4::<EmptyStruct>().unwrap();
    assert!(
        empty_struct.contains("export const EmptyStructSchema = z.strictObject({});"),
        "v4 empty struct should use z.strictObject({{}}), got: {empty_struct}"
    );

    let empty_variant = export_for_v4::<EmptyNamedVariant>().unwrap();
    assert!(
        empty_variant.contains("export const EmptyNamedVariantSchema = z.strictObject({});"),
        "v4 empty variant should use z.strictObject({{}}), got: {empty_variant}"
    );
}

#[test]
fn zod_v4_integer_types_use_z_int() {
    assert_eq!(inline_for_v4::<i8>().unwrap(), "z.int()");
    assert_eq!(inline_for_v4::<i16>().unwrap(), "z.int()");
    assert_eq!(inline_for_v4::<i32>().unwrap(), "z.int()");
    assert_eq!(inline_for_v4::<u8>().unwrap(), "z.int()");
    assert_eq!(inline_for_v4::<u16>().unwrap(), "z.int()");
    assert_eq!(inline_for_v4::<u32>().unwrap(), "z.int()");
}

#[test]
fn zod_v4_float_types_use_z_number() {
    assert_eq!(inline_for_v4::<f32>().unwrap(), "z.number()");
    assert_eq!(inline_for_v4::<f64>().unwrap(), "z.number()");
}

#[test]
fn zod_v4_bigint_number_uses_z_int() {
    let zod = Zod::default()
        .zod_version(ZodVersion::V4)
        .bigint(BigIntExportBehavior::Number);

    for_bigint_types!(T -> |_name| {
        let result = inline_for_with::<T>(&zod).unwrap();
        assert_eq!(result, "z.int()", "v4 BigInt-as-Number should use z.int()");
    });
}

#[test]
fn zod_v4_bigint_other_behaviors_unchanged() {
    let zod_string = Zod::default()
        .zod_version(ZodVersion::V4)
        .bigint(BigIntExportBehavior::String);
    let zod_bigint = Zod::default()
        .zod_version(ZodVersion::V4)
        .bigint(BigIntExportBehavior::BigInt);
    let zod_fail = Zod::default()
        .zod_version(ZodVersion::V4)
        .bigint(BigIntExportBehavior::Fail);

    for_bigint_types!(T -> |_name| {
        assert_eq!(inline_for_with::<T>(&zod_string).unwrap(), "z.string()");
        assert_eq!(inline_for_with::<T>(&zod_bigint).unwrap(), "z.bigint()");
        assert!(inline_for_with::<T>(&zod_fail).is_err());
    });
}

#[test]
fn zod_v3_integer_types_still_use_z_number() {
    let zod = Zod::default().zod_version(ZodVersion::V3);
    assert_eq!(inline_for_with::<i32>(&zod).unwrap(), "z.number()");
    assert_eq!(inline_for_with::<u16>(&zod).unwrap(), "z.number()");
}

#[test]
fn zod_v3_empty_shapes_still_use_strict() {
    let empty_struct = export_for::<EmptyStruct>().unwrap();
    assert!(empty_struct.contains("export const EmptyStructSchema = z.object({}).strict();"));

    let empty_variant = export_for::<EmptyNamedVariant>().unwrap();
    assert!(
        empty_variant.contains("export const EmptyNamedVariantSchema = z.object({}).strict();")
    );
}

#[test]
fn zod_v4_export_smoke() {
    #[derive(Type)]
    struct V4Demo {
        name: String,
        count: i32,
        ratio: f64,
        flag: bool,
        maybe: Option<String>,
    }

    let types = Types::default().register::<V4Demo>();
    let resolved = ResolvedTypes::from_resolved_types(types);

    let out = Zod::default()
        .zod_version(ZodVersion::V4)
        .bigint(BigIntExportBehavior::Number)
        .export(&resolved)
        .unwrap();

    assert!(out.contains("export const V4DemoSchema = z.object({\n\tname: z.string(),\n\tcount: z.int(),\n\tratio: z.number(),\n\tflag: z.boolean(),\n\tmaybe: z.string().nullable(),\n});"));
    assert!(out.contains("export type V4Demo = z.infer<typeof V4DemoSchema>;"));
}

#[test]
fn zod_version_defaults_to_v3() {
    let zod = Zod::default();
    assert_eq!(zod.zod_version, ZodVersion::V3);
}

// --- ts_enum enums still export as z.literal/z.union in Zod ---

#[derive(Type, Serialize)]
#[specta(collect = false, ts_enum)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[test]
fn zod_v3_enum_outputs_union_of_literals() {
    let types = Types::default().register::<Direction>();
    let resolved = ResolvedTypes::from_resolved_types(types);

    let out = Zod::default().export(&resolved).unwrap();

    assert!(
        out.contains("export const DirectionSchema = z.union([z.literal(\"Down\"), z.literal(\"Left\"), z.literal(\"Right\"), z.literal(\"Up\")]);"),
        "v3 string enum should output z.union of z.literal variants, got:\n{out}"
    );
    assert!(
        out.contains("export type Direction = z.infer<typeof DirectionSchema>;"),
        "got:\n{out}"
    );
}

#[test]
fn zod_v4_string_enum_uses_z_enum() {
    let types = Types::default().register::<Direction>();
    let resolved = ResolvedTypes::from_resolved_types(types);

    let out = Zod::default()
        .zod_version(ZodVersion::V4)
        .export(&resolved)
        .unwrap();

    assert!(
        out.contains(
            "export const DirectionSchema = z.enum([\"Down\", \"Left\", \"Right\", \"Up\"]);"
        ),
        "v4 string enum should use z.enum([...]), got:\n{out}"
    );
    assert!(
        out.contains("export type Direction = z.infer<typeof DirectionSchema>;"),
        "got:\n{out}"
    );
    assert!(
        !out.contains("z.literal"),
        "v4 string enum should not use z.literal, got:\n{out}"
    );
    assert!(
        !out.contains("z.union"),
        "v4 string enum should not use z.union, got:\n{out}"
    );
}

#[derive(Type, Serialize)]
#[specta(collect = false)]
enum MixedEnum {
    Unit,
    WithData(String),
}

#[test]
fn zod_v4_mixed_enum_still_uses_union() {
    let types = Types::default().register::<MixedEnum>();
    let resolved = ResolvedTypes::from_resolved_types(types);

    let out = Zod::default()
        .zod_version(ZodVersion::V4)
        .export(&resolved)
        .unwrap();

    assert!(
        out.contains("export const MixedEnumSchema = z.union([z.literal(\"Unit\"), z.string()]);"),
        "v4 mixed enum (unit + data variants) should still use z.union, got:\n{out}"
    );
}
