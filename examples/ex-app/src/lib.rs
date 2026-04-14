use ex_shared::{Pagination, Permission, Role, User};
use serde::{Deserialize, Serialize};
use specta::{Type, TypeCompanion, specta_const};

#[derive(Type, Serialize, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub debug: bool,
}

#[derive(Type, Serialize, Deserialize)]
pub struct UserListResponse {
    pub users: Vec<User>,
    pub pagination: Pagination,
}

#[derive(Type, TypeCompanion, Serialize, Deserialize)]
#[companion(
    derive_field(Type, Serialize, Deserialize),
    derive_value(Type, Serialize, Deserialize),
    serde_field(rename_all = "camelCase"),
    serde_value(rename_all = "camelCase", tag = "type", content = "data")
)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub role: Role,
    pub permission: Permission,
    pub other: Vec<serde_json::Value>,
}

#[derive(Type, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "t", content = "c")]
enum AdjacentlyTagged {
    UnitVariant,
    WithData(String),
    WithStruct { x: i32, y: i32 },
}

#[specta_const]
pub const CRATE_NAME1: &str = "app";

#[cfg(test)]
mod tests {
    use std::{fs, sync::LazyLock};

    use specta::export::{
        FolderGrouping, ImportStyle, IndexFileConfig, Layout, MultiFileConfig, SingleFileConfig,
    };

    static RESOLVED: LazyLock<specta::ResolvedTypes> = LazyLock::new(|| {
        let types = specta::collect_types();
        let constants = specta::collect_constants();
        specta_serde::apply(types)
            .expect("serde transformation failed")
            .with_constants(constants)
    });

    fn out_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("output")
    }

    /// Build a filesystem-safe name from a Layout for use as a subdirectory.
    fn layout_dir_name(layout: &Layout, namespaces: bool) -> String {
        let mut name = match layout {
            Layout::SingleFile(c) => {
                if c.module_prefix_names {
                    "single-prefixed".to_string()
                } else {
                    "single-flat".to_string()
                }
            }
            Layout::MultiFile(c) => {
                let fg = match &c.folder_grouping {
                    FolderGrouping::None => "fg-none",
                    FolderGrouping::ByDepth(d) => {
                        return format!(
                            "multi_fg-depth{d}_idx-{}_imp-{}",
                            idx_name(&c.index_files),
                            imp_name(&c.import_style)
                        );
                    }
                };
                format!(
                    "multi_{}_idx-{}_imp-{}",
                    fg,
                    idx_name(&c.index_files),
                    imp_name(&c.import_style),
                )
            }
        };
        if namespaces {
            name.push_str("_ns");
        }
        name
    }

    fn idx_name(idx: &IndexFileConfig) -> &'static str {
        match idx {
            IndexFileConfig::None => "none",
            IndexFileConfig::ReExportAll => "reexport",
        }
    }

    fn imp_name(imp: &ImportStyle) -> &'static str {
        match imp {
            ImportStyle::Namespace => "ns",
            ImportStyle::Named => "named",
        }
    }

    // -----------------------------------------------------------------------
    // All possible SingleFileConfig values
    // -----------------------------------------------------------------------
    fn single_file_configs() -> Vec<SingleFileConfig> {
        vec![
            SingleFileConfig {
                module_prefix_names: false,
            },
            SingleFileConfig {
                module_prefix_names: true,
            },
        ]
    }

    // -----------------------------------------------------------------------
    // All possible MultiFileConfig values (cartesian product)
    // -----------------------------------------------------------------------
    fn multi_file_configs() -> Vec<MultiFileConfig> {
        let folder_groupings = [FolderGrouping::None, FolderGrouping::ByDepth(1)];
        let index_files = [IndexFileConfig::None, IndexFileConfig::ReExportAll];
        let import_styles = [ImportStyle::Namespace, ImportStyle::Named];

        let mut configs = Vec::new();
        for fg in &folder_groupings {
            for idx in &index_files {
                for imp in &import_styles {
                    configs.push(MultiFileConfig {
                        folder_grouping: fg.clone(),
                        index_files: idx.clone(),
                        import_style: imp.clone(),
                    });
                }
            }
        }
        configs
    }

    // -----------------------------------------------------------------------
    // All Layout values
    // -----------------------------------------------------------------------
    fn all_layouts() -> Vec<Layout> {
        let mut layouts: Vec<Layout> = single_file_configs()
            .into_iter()
            .map(Layout::SingleFile)
            .collect();
        layouts.extend(multi_file_configs().into_iter().map(Layout::MultiFile));
        layouts
    }

    // =======================================================================
    // TypeScript
    // =======================================================================

    #[test]
    fn typescript_all_layouts() {
        let resolved = &*RESOLVED;
        let base = out_dir().join("ts");

        for layout in all_layouts() {
            let dir_name = layout_dir_name(&layout, false);

            match &layout {
                Layout::SingleFile(_) => {
                    let path = base.join(format!("{dir_name}.ts"));
                    let output = specta_typescript::Typescript::default()
                        .layout(layout.clone())
                        .export(resolved)
                        .expect(&format!("TS export failed for {dir_name}"));

                    fs::create_dir_all(path.parent().unwrap()).unwrap();
                    fs::write(&path, &output).unwrap();

                    assert!(
                        !output.is_empty(),
                        "TS single-file output should not be empty for {dir_name}"
                    );
                }
                Layout::MultiFile(_) => {
                    let path = base.join(&dir_name);
                    specta_typescript::Typescript::default()
                        .layout(layout.clone())
                        .export_to(&path, resolved)
                        .expect(&format!("TS export_to failed for {dir_name}"));

                    assert!(
                        path.is_dir(),
                        "TS multi-file output dir should exist for {dir_name}"
                    );
                }
            }
        }
    }

    #[test]
    fn typescript_namespaces() {
        let resolved = &*RESOLVED;
        let base = out_dir().join("ts");

        for config in single_file_configs() {
            let layout = Layout::SingleFile(config);
            let dir_name = layout_dir_name(&layout, true);
            let path = base.join(format!("{dir_name}.ts"));

            let output = specta_typescript::Typescript::default()
                .layout(layout)
                .namespaces(true)
                .export(resolved)
                .expect(&format!("TS namespace export failed for {dir_name}"));

            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, &output).unwrap();

            assert!(
                output.contains("namespace "),
                "TS namespace output should contain namespace for {dir_name}"
            );
        }
    }

    #[test]
    fn typescript_multi_file_errors_on_export_string() {
        let resolved = &*RESOLVED;
        for config in multi_file_configs() {
            let err = specta_typescript::Typescript::default()
                .layout(Layout::MultiFile(config))
                .export(resolved)
                .unwrap_err();
            assert!(
                err.to_string().contains("Unable to export"),
                "should error: {err}"
            );
        }
    }

    // =======================================================================
    // Zod
    // =======================================================================

    #[test]
    fn zod_all_layouts() {
        let resolved = &*RESOLVED;
        let base = out_dir().join("zod");

        for layout in all_layouts() {
            let dir_name = layout_dir_name(&layout, false);

            match &layout {
                Layout::SingleFile(_) => {
                    let path = base.join(format!("{dir_name}.ts"));
                    let output = specta_typescript::Zod::default()
                        .layout(layout.clone())
                        .export(resolved)
                        .expect(&format!("Zod export failed for {dir_name}"));

                    fs::create_dir_all(path.parent().unwrap()).unwrap();
                    fs::write(&path, &output).unwrap();

                    assert!(
                        output.contains("z."),
                        "Zod output should contain z. for {dir_name}"
                    );
                }
                Layout::MultiFile(_) => {
                    let path = base.join(&dir_name);
                    specta_typescript::Zod::default()
                        .layout(layout.clone())
                        .export_to(&path, resolved)
                        .expect(&format!("Zod export_to failed for {dir_name}"));

                    assert!(
                        path.is_dir(),
                        "Zod multi-file output dir should exist for {dir_name}"
                    );
                }
            }
        }
    }

    #[test]
    fn zod_multi_file_errors_on_export_string() {
        let resolved = &*RESOLVED;
        for config in multi_file_configs() {
            let err = specta_typescript::Zod::default()
                .layout(Layout::MultiFile(config))
                .export(resolved)
                .unwrap_err();
            assert!(
                err.to_string().contains("Unable to export"),
                "should error: {err}"
            );
        }
    }

    // =======================================================================
    // JSON Schema
    // =======================================================================

    #[test]
    fn jsonschema_all_layouts() {
        let resolved = &*RESOLVED;
        let base = out_dir().join("jsonschema");

        for layout in all_layouts() {
            let dir_name = layout_dir_name(&layout, false);

            match &layout {
                Layout::SingleFile(_) => {
                    let path = base.join(format!("{dir_name}.json"));
                    let output = specta_jsonschema::JsonSchema::default()
                        .layout(layout.clone())
                        .export(resolved)
                        .expect(&format!("JSON Schema export failed for {dir_name}"));

                    fs::create_dir_all(path.parent().unwrap()).unwrap();
                    fs::write(&path, &output).unwrap();

                    assert!(
                        output.contains("\"$schema\""),
                        "JSON Schema output should contain $schema for {dir_name}"
                    );
                }
                Layout::MultiFile(_) => {
                    let path = base.join(&dir_name);
                    specta_jsonschema::JsonSchema::default()
                        .layout(layout.clone())
                        .export_to(&path, resolved)
                        .expect(&format!("JSON Schema export_to failed for {dir_name}"));

                    assert!(
                        path.is_dir(),
                        "JSON Schema multi-file dir should exist for {dir_name}"
                    );
                }
            }
        }
    }

    #[test]
    fn jsonschema_multi_file_errors_on_export_string() {
        let resolved = &*RESOLVED;
        for config in multi_file_configs() {
            let err = specta_jsonschema::JsonSchema::default()
                .layout(Layout::MultiFile(config))
                .export(resolved)
                .unwrap_err();
            assert!(
                err.to_string().contains("MultiFile"),
                "should mention MultiFile: {err}"
            );
        }
    }
}
