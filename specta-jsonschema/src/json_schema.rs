use crate::{Error, Layout, SchemaVersion, primitives};
use serde_json::Value;
use specta::{ResolvedTypes, Types, datatype::NamedDataType};
use std::collections::BTreeMap;
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
}

impl Default for JsonSchema {
    fn default() -> Self {
        Self {
            schema_version: SchemaVersion::default(),
            layout: Layout::default(),
            title: None,
            description: None,
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

        let defs_key = self.schema_version.definitions_key();
        let mut root = serde_json::json!({
            "$schema": self.schema_version.uri(),
            defs_key: definitions,
        });

        if let Some(title) = &self.title {
            root.as_object_mut()
                .unwrap()
                .insert("title".to_string(), Value::String(title.clone()));
        }

        if let Some(description) = &self.description {
            root.as_object_mut().unwrap().insert(
                "description".to_string(),
                Value::String(description.clone()),
            );
        }

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
                let file_path = module_dir.join(filename);

                let mut root = serde_json::json!({
                    "$schema": self.schema_version.uri(),
                });

                if let Some(obj) = schema.as_object() {
                    for (k, v) in obj {
                        root.as_object_mut().unwrap().insert(k.clone(), v.clone());
                    }
                }

                std::fs::write(file_path, serde_json::to_string_pretty(&root)?)?;
            }
        }

        Ok(())
    }
}
