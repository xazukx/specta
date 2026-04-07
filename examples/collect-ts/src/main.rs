use serde::Serialize;
use specta::{ResolvedTypes, Type};

// -------------------------------------------------------
// Types — all auto-collected via `#[derive(Type)]`
// -------------------------------------------------------

#[derive(Type, Serialize)]
pub struct User {
    /// TODO: change to a string
    pub id: u32,
    pub name: String,
    pub email: String,
    pub role: Role,
    pub permission: Vec<Permission>,
}

#[derive(Type, Serialize)]
pub enum Role {
    Admin,
    Member,
    Guest,
}

#[derive(Type, Serialize)]
#[specta(ts_enum)]
#[serde(rename_all = "camelCase")]
/// What permissions you have
pub enum Permission {
    /// Can create anything
    Create,
    /// Can read anything
    Read,
    /// Can update anything
    Update,
    /// Can delete anything
    Delete,
}

#[derive(Type, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    UserCreated { user_id: u32, email: String },
    UserDeleted { user_id: u32 },
    SystemStarted,
}

#[derive(Type, Serialize)]
pub struct PaginatedResponse<T: Type> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
}

/// This type is excluded from auto-collection.
#[derive(Type, Serialize)]
#[specta(collect = false)]
pub struct InternalDebugInfo {
    pub trace_id: String,
}

// -------------------------------------------------------
// Constants — all auto-collected via `#[specta_const]`
// -------------------------------------------------------

/// Maximum number of login attempts before lockout.
#[specta::specta_const]
pub const MAX_LOGIN_ATTEMPTS: u32 = 5;

/// Current API version string.
#[specta::specta_const]
pub const API_VERSION: &str = "v2";

#[specta::specta_const]
pub const DEBUG_MODE: bool = false;

/// This constant is excluded from auto-collection.
#[specta::specta_const(collect = false)]
pub const INTERNAL_SECRET_SEED: u32 = 42;

fn main() {
    // collect_types() returns all types that have #[derive(Type)] (except collect = false)
    let types = specta::collect_types();
    // collect_constants() returns all #[specta_const] constants (except collect = false)
    let constants = specta::collect_constants();

    println!(
        "Collected {} types and {} constants\n",
        types.len(),
        constants.len()
    );

    let serde_resolved = specta_serde::apply(types)
        .expect("serde transformation failed")
        .with_constants(constants);

    let output_from_serde = specta_typescript::Typescript::default()
        .export(&serde_resolved)
        .expect("typescript export failed");

    println!("{output_from_serde}");

    // collect_types() returns all types that have #[derive(Type)] (except collect = false)
    let types = specta::collect_types();
    // collect_constants() returns all #[specta_const] constants (except collect = false)
    let constants = specta::collect_constants();

    println!(
        "Collected {} types and {} constants\n",
        types.len(),
        constants.len()
    );

    let resolved = ResolvedTypes::from_types_and_constants(types, constants);
    println!("\n----------------");

    let output_no_serde = specta_typescript::Typescript::default()
        .export(&resolved)
        .expect("typescript export failed");

    println!("{output_no_serde}");
}
