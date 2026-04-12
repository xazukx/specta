use serde::Serialize;
use specta::TypeCompanion;
use specta::companion::{CompanionField, CompanionValue, TypeCompanion};

// ─── Struct example: full companion generation ──────────────────────────────

#[derive(Clone, Serialize, TypeCompanion)]
#[companion(derive_value(Eq))]
#[serde(rename_all = "camelCase")]
struct UserProfile {
    id: u64,
    #[companion(title = "Name")]
    #[serde(rename = "user_name")]
    username: String,
    #[companion(skip)]
    password_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
}

// ─── Enum example: variant names only ───────────────────────────────────────

#[allow(dead_code)]
#[derive(Serialize, TypeCompanion)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Status {
    Active,
    Inactive,
    #[companion(skip)]
    Internal,
    #[serde(rename = "on_hold")]
    OnHold,
}

// ─── Generic struct example ─────────────────────────────────────────────────

#[derive(Clone, TypeCompanion)]
struct Wrapper<T: Clone> {
    label: String,
    inner: T,
}

// ─── Generic programming with the TypeCompanion trait ────────────────────────

fn print_fields<F: CompanionField + 'static, V: CompanionValue, T: TypeCompanion<F, V>>(item: &T) {
    println!("Fields:");
    for field in T::fields() {
        let value = item.value(*field);
        println!(
            "  {} ({}): {}",
            field.name(),
            field.type_str(),
            value.field_name()
        );
    }
}

fn main() {
    // --- Struct companion usage ---
    let mut user = UserProfile {
        id: 1,
        username: "alice".into(),
        password_hash: "secret".into(),
        display_name: Some("Alice".into()),
    };

    // FIELD_NAMES reflects serde serialization names
    println!("Field names: {:?}", UserProfile::FIELD_NAMES);
    // Outputs: ["id", "user_name", "displayName"]

    // Access field values dynamically
    let name_val = user.value(UserProfileField::UserName);
    println!("Username value: {:?}", name_val);

    // Update a field dynamically
    user.update(UserProfileValue::UserName("bob".into()));
    println!("Updated username: {}", user.username);

    // Iterate all values
    for val in user.as_values() {
        println!("  {}: {:?}", val.field_name(), val.type_name());
    }

    // CompanionField metadata
    println!("UserName title: {}", UserProfileField::UserName.title());

    // Display and FromStr
    let field_str = UserProfileField::Id.to_string();
    let parsed: UserProfileField = field_str.parse().expect("parse failed");
    assert_eq!(parsed, UserProfileField::Id);

    // Generic programming
    print_fields::<UserProfileField, UserProfileValue, _>(&user);

    // --- Enum companion usage ---
    println!("\nStatus variant names: {:?}", Status::VARIANT_NAMES);
    // Outputs: ["ACTIVE", "INACTIVE", "on_hold"]

    // --- Generic struct usage ---
    let w = Wrapper {
        label: "example".into(),
        inner: 42i32,
    };
    println!("\nWrapper fields: {:?}", Wrapper::<i32>::FIELD_NAMES);
    let v = w.value(WrapperField::Inner);
    println!("Inner value: {:?}", v.field_name());
}
