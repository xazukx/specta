mod script;
use serde::{Deserialize, Serialize};
use specta::{Type, specta_const};

use crate::script::SqlScriptSource;

#[derive(Type, Serialize, Deserialize)]
pub struct UserId(pub u32);

#[derive(Type, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub role: Role,
    pub script: SqlScriptSource,
}

#[derive(Type, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Role {
    Admin,
    Member,
    Guest,
}

#[derive(Type, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
}

#[derive(Type, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[specta(ts_enum)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    ReadAll,
    WriteAll,
}

#[specta_const]
pub const CRATE_NAME2: &str = "shared";
