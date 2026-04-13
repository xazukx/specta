use crate::{Error, SchemaVersion, primitives};
use serde_json::Value;
use specta::{
    ResolvedTypes, Types,
    datatype::NamedDataType,
    export::{ExportLanguage, FolderGrouping, ImportInfo, Layout, ModuleRenderResult, PathResolver},
};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// JSON Schema exporter configuration
#[derive(Debug, Clone)]
pub struct JsonSchema {
    /// JSON Schema version to use
    pub schema_version: SchemaVersion,
    /// Layout for output organization.
    /// Use `Layout::default()` for single-file output, `Layout::multi_file()` for per-type files.
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

        match &self.layout {
            Layout::SingleFile(_) => {
                format!("#/{}/{}", self.schema_version.definitions_key(), type_name)
            }
            Layout::MultiFile(_) => {
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
        match &self.layout {
            Layout::SingleFile(_) => self.export_single_file(types),
            Layout::MultiFile(_) => Err(Error::ConversionError(
                "Use export_to() for MultiFile layout".to_string(),
            )),
        }
    }

    /// Export resolved types to file or directory.
    pub fn export_to(&self, path: impl AsRef<Path>, resolved: &ResolvedTypes) -> Result<(), Error> {
        let path = path.as_ref();
        let types = resolved.as_types();

        match &self.layout {
            Layout::SingleFile(_) => {
                let json = self.export_single_file(types)?;
                std::fs::write(path, serde_json::to_string_pretty(&json)?)?;
                Ok(())
            }
            Layout::MultiFile(_) => self.export_files(path, types),
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
        match &self.layout {
            Layout::SingleFile(_) => self.export_single_file(types),
            Layout::MultiFile(_) => Err(Error::ConversionError(
                "Use export_to() for MultiFile layout".to_string(),
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

        let folder_grouping = match &self.layout {
            Layout::MultiFile(config) => &config.folder_grouping,
            _ => &FolderGrouping::None,
        };

        for ndt in types
            .into_sorted_iter()
            .filter(|ndt| ndt.requires_reference(types))
        {
            let schema = primitives::export(self, types, &ndt)?;
            let module_path_str = ndt.module_path().to_string();

            // Compute directory based on FolderGrouping
            let module_dir = if module_path_str.is_empty() {
                base_path.to_path_buf()
            } else {
                match folder_grouping {
                    FolderGrouping::None => {
                        base_path.join(module_path_str.replace("::", "/"))
                    }
                    FolderGrouping::ByDepth(depth) => {
                        let segments: Vec<&str> = module_path_str.split("::").collect();
                        let folder_segments = if segments.len() <= *depth {
                            &segments[..]
                        } else {
                            &segments[..*depth]
                        };
                        if folder_segments.is_empty() {
                            base_path.to_path_buf()
                        } else {
                            base_path.join(folder_segments.join("/"))
                        }
                    }
                }
            };

            std::fs::create_dir_all(&module_dir)?;

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

        Ok(())
    }
}

// --- ExportLanguage trait implementation ---

impl ExportLanguage for JsonSchema {
    type Error = Error;

    fn file_extension(&self) -> &str {
        "schema.json"
    }

    fn index_file_stem(&self) -> Option<&str> {
        None // JSON Schema doesn't use barrel files
    }

    fn render_file_header(&self) -> String {
        String::new() // JSON has no header comments
    }

    fn render_module_types(
        &self,
        types: &Types,
        module_types: &[&NamedDataType],
        _indent: &str,
    ) -> Result<ModuleRenderResult, Error> {
        let mut body = String::new();
        let mut exports = HashMap::new();

        for ndt in module_types {
            if !ndt.requires_reference(types) {
                continue;
            }

            let schema = primitives::export(self, types, ndt)?;
            let name = ndt.name().to_string();

            let mut root = serde_json::json!({
                "$schema": self.schema_version.uri(),
            });

            let root_obj = root.as_object_mut().unwrap();

            if let Some(base) = &self.base_uri {
                let base = base.trim_end_matches('/');
                let filename = format!("{}.schema.json", name);
                root_obj.insert(
                    "$id".to_string(),
                    Value::String(format!("{}/{}", base, filename)),
                );
            }

            if let Some(obj) = schema.as_object() {
                for (k, v) in obj {
                    root_obj.insert(k.clone(), v.clone());
                }
            }

            if !body.is_empty() {
                body.push('\n');
            }
            body.push_str(&serde_json::to_string_pretty(&root)?);

            exports.insert(name, ndt.location());
        }

        Ok(ModuleRenderResult {
            body,
            exports,
            referenced_modules: BTreeMap::new(), // JSON Schema uses inline $ref
        })
    }

    fn render_imports(
        &self,
        _from_module_path: &str,
        _imports: &BTreeMap<String, ImportInfo>,
        _path_resolver: &PathResolver,
    ) -> Result<String, Error> {
        Ok(String::new()) // JSON Schema uses inline $ref, not import statements
    }

    fn exported_type_name(&self, _layout: &Layout, ndt: &NamedDataType) -> Cow<'static, str> {
        ndt.name().clone()
    }
}
