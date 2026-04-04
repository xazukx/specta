use std::io;

/// Errors that can occur during JSON Schema export or import.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// File system I/O error.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// The specified schema version string is not recognized.
    #[error("Invalid schema version: {0}")]
    InvalidSchemaVersion(String),

    /// Two types share the same name, causing a collision in the definitions map.
    #[error("Duplicate type name '{name}' at {location1} and {location2}")]
    DuplicateTypeName {
        /// The conflicting type name.
        name: String,
        /// Source location of the first type.
        location1: String,
        /// Source location of the second type.
        location2: String,
    },

    /// A type name is not valid for use in JSON Schema output.
    #[error("Invalid type name '{name}' at {path}")]
    InvalidTypeName {
        /// The invalid name.
        name: String,
        /// Module path where the type was defined.
        path: String,
    },

    /// General schema conversion failure.
    #[error("Unable to convert schema: {0}")]
    ConversionError(String),

    /// A Specta DataType variant that is not supported by the JSON Schema exporter.
    #[error("Unsupported DataType: {0}")]
    UnsupportedDataType(String),

    /// A type reference could not be resolved.
    #[error("Invalid reference: {0}")]
    InvalidReference(String),
}
