use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;

/// Distinguishes between a manually written SQL script and one loaded from an
/// external source file.
#[derive(Type, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SqlScriptSource {
    Unknown,
    /// Source metadata for scripts imported from database objects.
    #[serde(rename_all = "camelCase")]
    LiveDatabase {
        database_profile_id: String,
        database_object_name: String,
    },
    /// Source metadata for scripts imported from SQL statements.
    #[serde(rename_all = "camelCase")]
    SqlImport {
        object_name: String,
    },
    /// The destination path for a manually written script.
    #[serde(rename_all = "camelCase")]
    Manual {
        file_path_template: String,
    },
    /// The path for a script that is loaded from an external source file.
    #[serde(rename_all = "camelCase")]
    LoadedFromFile {
        source_file_path: PathBuf,
        loaded_location: String,
    },
}
