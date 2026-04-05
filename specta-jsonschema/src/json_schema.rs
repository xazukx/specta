use crate::{Error, Layout, SchemaVersion, primitives};
use serde_json::Value;
use specta::{ResolvedTypes, Types, datatype::NamedDataType};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// JSON Schema exporter configuration
#[derive(Debug, Clone)]
pub struct JsonSchema {
    /// JSON Schema version to use
    pub schema_version: SchemaVersion,
    /// Layout for output organization
    pub layout: Layout,
    /// Optional title for the root schema
    pub title: Option<String>,
    /// Optional description for the root schema
    pub description: Option<String>,
    /// Optional base URI for `$id` on the root schema document.
    /// When set, the root schema gets `"$id": "<base_uri>"` and in Files layout
    /// references use this as the base for absolute URIs.
    pub base_uri: Option<String>,
    /// Use `oneOf` instead of `anyOf` for enum union schemas.
    /// `oneOf` provides stricter validation (exactly one must match).
    pub use_one_of: bool,
    /// External `$ref` overrides. Maps type names to external URIs.
    /// When a type name appears in this map, references to it use the
    /// external URI instead of local `#/definitions/...` or file paths.
    pub external_refs: HashMap<String, String>,
}

impl Default for JsonSchema {
    fn default() -> Self {
        Self {
            schema_version: SchemaVersion::default(),
            layout: Layout::default(),
            title: None,
            description: None,
            base_uri: None,
            use_one_of: true,
            external_refs: HashMap::new(),
        }
    }
}

impl JsonSchema {
    /// Create a new JsonSchema exporter with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set JSON Schema version
    pub fn schema_version(mut self, version: SchemaVersion) -> Self {
        self.schema_version = version;
        self
    }

    /// Set output layout
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    /// Set root schema title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set root schema description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set a base URI for the root schema `$id`.
    ///
    /// This also controls how references are generated in `Files` layout:
    /// references become absolute URIs like `{base_uri}/{TypeName}`.
    pub fn base_uri(mut self, uri: impl Into<String>) -> Self {
        self.base_uri = Some(uri.into());
        self
    }

    /// Use `oneOf` instead of `anyOf` for enum union schemas.
    pub fn one_of(mut self, enabled: bool) -> Self {
        self.use_one_of = enabled;
        self
    }

    /// Register an external `$ref` URI for a type name.
    ///
    /// When this type is referenced, the external URI is used instead of
    /// a local `#/definitions/...` pointer.
    pub fn external_ref(mut self, type_name: impl Into<String>, uri: impl Into<String>) -> Self {
        self.external_refs.insert(type_name.into(), uri.into());
        self
    }

    /// Build the `$ref` string for a given type name, respecting layout,
    /// base_uri, and external_refs configuration.
    /// Build the `$ref` URI for a given type name, respecting layout,
    /// `base_uri`, and `external_refs` configuration.
    pub fn build_ref(&self, type_name: &str) -> String {
        if let Some(uri) = self.external_refs.get(type_name) {
            return uri.clone();
        }

        match self.layout {
            Layout::SingleFile => {
                format!("#/{}/{}", self.schema_version.definitions_key(), type_name)
            }
            Layout::Files => {
                if let Some(base) = &self.base_uri {
                    let base = base.trim_end_matches('/');
                    format!("{}/{}.schema.json", base, type_name)
                } else {
                    format!("./{}.schema.json", type_name)
                }
            }
        }
    }

    /// Export resolved types to JSON Schema as a JSON string.
    ///
    /// Use this after calling `specta_serde::apply()` to get correct serde
    /// representations (enum tagging, renames, etc).
    pub fn export(&self, resolved: &ResolvedTypes) -> Result<String, Error> {
        let value = self.export_as_value(resolved)?;
        Ok(serde_json::to_string_pretty(&value)?)
    }

    /// Export resolved types to JSON Schema as a `serde_json::Value`.
    pub fn export_as_value(&self, resolved: &ResolvedTypes) -> Result<Value, Error> {
        let types = resolved.as_types();
        match self.layout {
            Layout::SingleFile => self.export_single_file(types),
            Layout::Files => Err(Error::ConversionError(
                "Use export_to() for Files layout".to_string(),
            )),
        }
    }

    /// Export resolved types to file or directory.
    pub fn export_to(&self, path: impl AsRef<Path>, resolved: &ResolvedTypes) -> Result<(), Error> {
        let path = path.as_ref();
        let types = resolved.as_types();

        match self.layout {
            Layout::SingleFile => {
                let json = self.export_single_file(types)?;
                std::fs::write(path, serde_json::to_string_pretty(&json)?)?;
                Ok(())
            }
            Layout::Files => self.export_files(path, types),
        }
    }

    /// Export raw types (without serde transformation) as a JSON string.
    ///
    /// Prefer [`export`](Self::export) with `specta_serde::apply()` for types
    /// that use serde attributes.
    pub fn export_raw(&self, types: &Types) -> Result<String, Error> {
        let value = self.export_raw_as_value(types)?;
        Ok(serde_json::to_string_pretty(&value)?)
    }

    /// Export raw types (without serde transformation) as a `serde_json::Value`.
    pub fn export_raw_as_value(&self, types: &Types) -> Result<Value, Error> {
        match self.layout {
            Layout::SingleFile => self.export_single_file(types),
            Layout::Files => Err(Error::ConversionError(
                "Use export_to() for Files layout".to_string(),
            )),
        }
    }

    fn export_single_file(&self, types: &Types) -> Result<Value, Error> {
        let mut definitions = BTreeMap::new();

        for ndt in types
            .into_sorted_iter()
            .filter(|ndt| ndt.requires_reference(types))
        {
            let schema = primitives::export(self, types, &ndt)?;
            let name = ndt.name().to_string();
            definitions.insert(name, schema);
        }

        let mut root = serde_json::json!({
            "$schema": self.schema_version.uri(),
        });

        let root_obj = root.as_object_mut().unwrap();

        if let Some(base_uri) = &self.base_uri {
            root_obj.insert("$id".to_string(), Value::String(base_uri.clone()));
        }

        if let Some(title) = &self.title {
            root_obj.insert("title".to_string(), Value::String(title.clone()));
        }

        if let Some(description) = &self.description {
            root_obj.insert(
                "description".to_string(),
                Value::String(description.clone()),
            );
        }

        let defs_key = self.schema_version.definitions_key();
        root_obj.insert(defs_key.into(), serde_json::to_value(definitions).unwrap());

        Ok(root)
    }

    fn export_files(&self, base_path: &Path, types: &Types) -> Result<(), Error> {
        std::fs::create_dir_all(base_path)?;

        let mut by_module: BTreeMap<String, Vec<NamedDataType>> = BTreeMap::new();

        for ndt in types
            .into_sorted_iter()
            .filter(|ndt| ndt.requires_reference(types))
        {
            let module = ndt.module_path().to_string().replace("::", "/");
            by_module.entry(module).or_default().push(ndt.clone());
        }

        for (module, ndts) in by_module {
            let module_dir = if module.is_empty() {
                base_path.to_path_buf()
            } else {
                base_path.join(&module)
            };

            std::fs::create_dir_all(&module_dir)?;

            for ndt in &ndts {
                let schema = primitives::export(self, types, ndt)?;
                let filename = format!("{}.schema.json", ndt.name());
                let file_path = module_dir.join(&filename);

                let mut root = serde_json::json!({
                    "$schema": self.schema_version.uri(),
                });

                let root_obj = root.as_object_mut().unwrap();

                // Add $id for each file
                if let Some(base) = &self.base_uri {
                    let base = base.trim_end_matches('/');
                    root_obj.insert(
                        "$id".to_string(),
                        Value::String(format!("{}/{}", base, filename)),
                    );
                }

                // Merge in the type's schema properties
                if let Some(obj) = schema.as_object() {
                    for (k, v) in obj {
                        root_obj.insert(k.clone(), v.clone());
                    }
                }

                std::fs::write(file_path, serde_json::to_string_pretty(&root)?)?;
            }
        }

        Ok(())
    }
}
