use serde::Serialize;
use specta::TypeCompanion;
use specta::companion::{CompanionField, CompanionValue, TypeCompanion};
use std::convert::TryFrom;

// ──────────────────────────────────────────────────────────────────────────────
// Basic struct
// ─────────────────────────────��──────────────────────────────────��─────────────

#[derive(Clone, TypeCompanion)]
struct Basic {
    id: u64,
    name: String,
}

#[test]
fn basic_field_names() {
    assert_eq!(Basic::FIELD_NAMES, &["id", "name"]);
    assert_eq!(Basic::field_names(), &["id", "name"]);
}

#[test]
fn basic_fields() {
    let fields = Basic::fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name(), "id");
    assert_eq!(fields[1].name(), "name");
}

#[test]
fn basic_value_and_update() {
    let mut b = Basic {
        id: 1,
        name: "Alice".into(),
    };

    let v = b.value(BasicField::Id);
    assert_eq!(v.field_name(), "id");
    assert_eq!(v.type_name(), "u64");

    b.update(BasicValue::Name("Bob".into()));
    assert_eq!(b.name, "Bob");
}

#[test]
fn basic_as_values() {
    let b = Basic {
        id: 42,
        name: "Test".into(),
    };
    let values = b.as_values();
    assert_eq!(values.len(), 2);
}

#[test]
fn basic_display_and_fromstr() {
    assert_eq!(BasicField::Id.to_string(), "id");
    assert_eq!(BasicField::Name.to_string(), "name");

    assert_eq!("id".parse::<BasicField>(), Ok(BasicField::Id));
    assert_eq!("name".parse::<BasicField>(), Ok(BasicField::Name));
    assert!("unknown".parse::<BasicField>().is_err());
}

#[test]
fn basic_try_from() {
    let v = BasicValue::Id(42);
    let id: u64 = v.try_into().unwrap();
    assert_eq!(id, 42);

    let v = BasicValue::Name("hello".into());
    let name: String = v.try_into().unwrap();
    assert_eq!(name, "hello");
}

// ──────────────────────────────────���───────────────────────────────────────────
// Serde rename_all
// ───────────────────���─────────────────────────────���────────────────────────────

#[derive(Clone, Serialize, TypeCompanion)]
#[serde(rename_all = "camelCase")]
struct CamelStruct {
    user_name: String,
    display_name: Option<String>,
}

#[test]
fn serde_rename_all_camel() {
    assert_eq!(CamelStruct::FIELD_NAMES, &["userName", "displayName"]);
    assert_eq!(CamelStructField::UserName.name(), "userName");
    assert_eq!(CamelStructField::DisplayName.name(), "displayName");
}

// ─────────��────────────────────────────────────────────────────────────────────
// Serde rename on individual fields
// ──────────────────────��───────────────────────────────────────────────────────

#[derive(Clone, Serialize, TypeCompanion)]
struct RenameField {
    id: u64,
    #[serde(rename = "user_name")]
    username: String,
}

#[test]
fn serde_rename_field() {
    assert_eq!(RenameField::FIELD_NAMES, &["id", "user_name"]);
    // Variant is PascalCase of the serialized name
    assert_eq!(RenameFieldField::UserName.name(), "user_name");
}

// ──���────────────────────���─────────────────────────────────���────────────────────
// Combined rename_all + field rename
// ──���────────────────────────────────────────────────────��──────────────────────

#[derive(Clone, Serialize, TypeCompanion)]
#[serde(rename_all = "camelCase")]
struct CombinedRename {
    id: u64,
    #[serde(rename = "user_name")]
    username: String,
    display_name: Option<String>,
}

#[test]
fn combined_rename() {
    assert_eq!(
        CombinedRename::FIELD_NAMES,
        &["id", "user_name", "displayName"]
    );
}

// ────��─────────────────────���─────────────────────────────────���─────────────────
// companion(skip)
// ��─────────────────────���─────────────────────────────���─────────────────────────

#[derive(Clone, TypeCompanion)]
struct WithSkip {
    id: u64,
    #[companion(skip)]
    password_hash: String,
    name: String,
}

#[test]
fn companion_skip() {
    assert_eq!(WithSkip::FIELD_NAMES, &["id", "name"]);
    assert_eq!(WithSkip::fields().len(), 2);
}

// ────���───────────────────────────────���─────────────────────────────────────────
// serde(skip) also skips companion
// ───────────────────────────��────────────────────────────��─────────────────────

#[derive(Clone, Serialize, TypeCompanion)]
struct WithSerdeSkip {
    id: u64,
    #[serde(skip)]
    internal: String,
    name: String,
}

#[test]
fn serde_skip_also_skips_companion() {
    assert_eq!(WithSerdeSkip::FIELD_NAMES, &["id", "name"]);
}

// ────────────────────��─────────────────────────────────���───────────────────────
// Title and order metadata
// ───────���──────────────────��───────────────────────────────────────────────────

#[derive(Clone, TypeCompanion)]
struct WithMeta {
    #[companion(title = "Identifier", order = 1)]
    id: u64,
    #[companion(title = "Full Name", order = 2)]
    name: String,
}

#[test]
fn title_and_order() {
    assert_eq!(WithMetaField::Id.title(), "Identifier");
    assert_eq!(WithMetaField::Id.order(), 1);
    assert_eq!(WithMetaField::Name.title(), "Full Name");
    assert_eq!(WithMetaField::Name.order(), 2);
}

// ──��──────────────────────────────��────────────────────────────────────────────
// TypeCompanion trait for generic programming
// ─��────────────────────��───────────────────────────────────────────────────────

fn count_fields<F: Copy + 'static, V, T: TypeCompanion<F, V>>() -> usize {
    T::fields().len()
}

#[test]
fn type_companion_trait() {
    assert_eq!(count_fields::<BasicField, BasicValue, Basic>(), 2);
}

// ───��───────────────────────────────────────────────��──────────────────────────
// Custom method names
// ────────────────────────��────────────────────────────────���────────────────────

#[derive(Clone, TypeCompanion)]
#[companion(
    value_fn = "get_value",
    update_fn = "set_value",
    fields_fn = "all_fields"
)]
struct CustomMethods {
    x: i32,
}

#[test]
fn custom_method_names() {
    let mut c = CustomMethods { x: 10 };
    let v = c.get_value(CustomMethodsField::X);
    assert!(matches!(v, CustomMethodsValue::X(10)));

    c.set_value(CustomMethodsValue::X(20));
    assert_eq!(c.x, 20);

    let fields = CustomMethods::all_fields();
    assert_eq!(fields.len(), 1);
}

// ───────────────────────────��─────────────────────────��────────────────────────
// derive_value / derive_field extra derives
// ��─────────────────────────────��───────────────────────────────────────────────

#[derive(Clone, TypeCompanion)]
#[companion(derive_value(Eq))]
struct WithExtraDerives {
    id: u64,
}

#[test]
fn extra_derives() {
    // Eq is derived on the value enum
    let v1 = WithExtraDerivesValue::Id(1);
    let v2 = WithExtraDerivesValue::Id(1);
    assert_eq!(v1, v2);
}

// ─────���────────────────────────────���──────────────────────────��────────────────
// Enum — VARIANT_NAMES
// ─────────��──────────────────────────────────────────────────────────────────���─

#[derive(TypeCompanion)]
enum Status {
    Active,
    Inactive,
    OnHold,
}

#[test]
fn enum_variant_names() {
    assert_eq!(Status::VARIANT_NAMES, &["Active", "Inactive", "OnHold"]);
    assert_eq!(Status::variant_names(), &["Active", "Inactive", "OnHold"]);
}

// ───────────────────────────────────────────────���──────────────────────────────
// Enum with serde rename_all
// ─────���────────────────────────────────────────────────��───────────────────────

#[derive(Serialize, TypeCompanion)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ScreamingEnum {
    Active,
    Inactive,
    OnHold,
}

#[test]
fn enum_serde_rename_all() {
    assert_eq!(
        ScreamingEnum::VARIANT_NAMES,
        &["ACTIVE", "INACTIVE", "ON_HOLD"]
    );
}

// ─���─────────────────────���────────────────────────────────���─────────────────────
// Enum with serde rename on individual variant
// ────────────────────────────────────────────────────────���─────────────────────

#[derive(Serialize, TypeCompanion)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum MixedEnum {
    Active,
    Inactive,
    #[serde(rename = "on_hold")]
    OnHold,
}

#[test]
fn enum_serde_rename_variant() {
    assert_eq!(MixedEnum::VARIANT_NAMES, &["ACTIVE", "INACTIVE", "on_hold"]);
}

// ────────────────────────────���─────────────────────────────────────────────────
// Enum with companion(skip)
// ───────────────────���────────────────────────────────────��─────────────────────

#[derive(TypeCompanion)]
enum StatusWithSkip {
    Active,
    Inactive,
    #[companion(skip)]
    Internal,
}

#[test]
fn enum_companion_skip() {
    assert_eq!(StatusWithSkip::VARIANT_NAMES, &["Active", "Inactive"]);
}

// ──────────────────────────��───────────────────────────────────────────────────
// Option<T> fields handled naturally
// ───��──────────────────────��───────────────────────────────────────────────────

#[derive(Clone, TypeCompanion)]
struct WithOptional {
    id: u64,
    nickname: Option<String>,
}

#[test]
fn optional_field() {
    let mut s = WithOptional {
        id: 1,
        nickname: None,
    };
    s.update(WithOptionalValue::Nickname(Some("nick".into())));
    assert_eq!(s.nickname, Some("nick".into()));
}

// ─��────────────────────────────────��───────────────────────────���───────────────
// CompanionValue::field() method
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn value_field_method() {
    let v = BasicValue::Id(1);
    assert_eq!(v.field(), BasicField::Id);

    let v = BasicValue::Name("test".into());
    assert_eq!(v.field(), BasicField::Name);
}

// ───────────��──────────────────────────────���───────────────────────────────────
// Visibility: pub struct generates pub enums
// ─────────────────────���─────────────────────────────────��──────────────────────

mod inner {
    use specta::TypeCompanion;

    #[derive(Clone, TypeCompanion)]
    pub struct PubStruct {
        pub x: i32,
    }
}

#[test]
fn visibility_inherited() {
    // PubStructField and PubStructValue should be accessible from outside the module
    let _f = inner::PubStructField::X;
    let _v = inner::PubStructValue::X(42);
}

// ──────────────────────────────────────────────────────────────────────────────
// TryFrom<Value> failure case
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn try_from_failure() {
    let v = BasicValue::Name("hello".into());
    let result = u64::try_from(v);
    assert!(result.is_err());
    // The error contains the original value
    let err = result.unwrap_err();
    assert!(matches!(err, BasicValue::Name(_)));
}

// ──────────────────────────────────────────────────────────────────────────────
// TryFrom<(Field, T)> for Value
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn try_from_tuple() {
    // Construct a value from (Field, T) tuple
    let v = BasicValue::try_from((BasicField::Id, 99u64));
    assert!(v.is_ok());
    assert!(matches!(v.unwrap(), BasicValue::Id(99)));

    // Wrong field type combination fails
    let v = BasicValue::try_from((BasicField::Name, 99u64));
    assert!(v.is_err());
}

// ──────────────────────────────────────────────────────────────────────────────
// Generic struct support
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Clone, TypeCompanion)]
struct Wrapper<T: Clone> {
    label: String,
    inner: T,
}

#[test]
fn generic_struct_field_names() {
    assert_eq!(Wrapper::<i32>::FIELD_NAMES, &["label", "inner"]);
}

#[test]
fn generic_struct_value_and_update() {
    let mut w = Wrapper {
        label: "test".into(),
        inner: 42i32,
    };

    let v = w.value(WrapperField::Inner);
    assert!(matches!(v, WrapperValue::Inner(42)));

    w.update(WrapperValue::Inner(100));
    assert_eq!(w.inner, 100);
}

#[test]
fn generic_struct_as_values() {
    let w = Wrapper {
        label: "x".into(),
        inner: true,
    };
    let vals = w.as_values();
    assert_eq!(vals.len(), 2);
}

#[test]
fn generic_struct_type_companion_trait() {
    // TypeCompanion trait works with generic struct
    fn get_first_value<F: Copy + 'static, V, T: TypeCompanion<F, V>>(t: &T) -> V {
        let fields = T::fields();
        t.value(fields[0])
    }

    let w = Wrapper {
        label: "hello".into(),
        inner: 5u32,
    };
    let v = get_first_value::<WrapperField, WrapperValue<u32>, _>(&w);
    assert!(matches!(v, WrapperValue::Label(_)));
}

// ──────────────────────────────────────────────────────────────────────────────
// FromStr accepts both serialized name and PascalCase variant name
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn fromstr_accepts_variant_name() {
    // For CamelStruct, serialized name is "userName" and variant name is "UserName"
    assert_eq!(
        "userName".parse::<CamelStructField>(),
        Ok(CamelStructField::UserName)
    );
    assert_eq!(
        "UserName".parse::<CamelStructField>(),
        Ok(CamelStructField::UserName)
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// serde_field / serde_value attributes
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, TypeCompanion)]
#[companion(
    derive_field(Serialize),
    derive_value(Serialize),
    serde_field(rename_all = "SCREAMING_SNAKE_CASE"),
    serde_value(rename_all = "snake_case")
)]
struct WithSerdeAttrs {
    id: u64,
    name: String,
}

#[test]
fn serde_attrs_on_generated_enums() {
    // serde_field(rename_all = "SCREAMING_SNAKE_CASE") makes field enum serialize as SCREAMING_SNAKE_CASE.
    let field_json = serde_json::to_string(&WithSerdeAttrsField::Id).unwrap();
    assert_eq!(field_json, "\"ID\"");

    let field_json = serde_json::to_string(&WithSerdeAttrsField::Name).unwrap();
    assert_eq!(field_json, "\"NAME\"");

    // serde_value(rename_all = "snake_case") makes value enum serialize as snake_case.
    let val_json = serde_json::to_string(&WithSerdeAttrsValue::Id(42)).unwrap();
    assert!(val_json.contains("\"id\""));
}
