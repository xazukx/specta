use ex_shared::{Pagination, Permission, Role, User};
use serde::{Deserialize, Serialize};
use specta::{Type, specta_const};

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

#[derive(Type, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub role: Role,
    pub permission: Permission,
}

#[derive(Type, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum AdjacentlyTagged {
    UnitVariant,
    WithData(String),
    WithStruct { x: i32, y: i32 },
}

#[specta_const]
pub const CRATE_NAME: &str = "app";

#[cfg(test)]
mod tests {
    use std::fs;

    use specta::ResolvedTypes;

    #[test]
    fn export_to_typescript_files() {
        let types = specta::collect_types();
        let constants = specta::collect_constants();

        let resolved = specta_serde::apply(types)
            .expect("serde transformation failed")
            .with_constants(constants);

        let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("output");

        specta_typescript::Typescript::default()
            .layout(specta_typescript::Layout::Files)
            .export_to(&out_dir, &resolved)
            .expect("typescript export failed");

        // Read the generated files
        let shared_path = out_dir.join("ex_shared.ts");
        let app_path = out_dir.join("ex_app.ts");

        assert!(shared_path.exists(), "ex_shared.ts should be generated");
        assert!(app_path.exists(), "ex_app.ts should be generated");

        let shared_content = fs::read_to_string(&shared_path).unwrap();
        let app_content = fs::read_to_string(&app_path).unwrap();

        // Shared types should be in ex_shared.ts
        assert!(
            shared_content.contains("export type User"),
            "ex_shared.ts should contain User type"
        );
        assert!(
            shared_content.contains("export type UserId"),
            "ex_shared.ts should contain UserId type"
        );
        assert!(
            shared_content.contains("export type Role"),
            "ex_shared.ts should contain Role type"
        );
        assert!(
            shared_content.contains("export type Pagination"),
            "ex_shared.ts should contain Pagination type"
        );

        // App types should be in ex_app.ts
        assert!(
            app_content.contains("export type AppConfig"),
            "ex_app.ts should contain AppConfig type"
        );
        assert!(
            app_content.contains("export type UserListResponse"),
            "ex_app.ts should contain UserListResponse type"
        );
        assert!(
            app_content.contains("export type CreateUserRequest"),
            "ex_app.ts should contain CreateUserRequest type"
        );

        // ex_app.ts should use a value import (not `import type`) because
        // ex_shared contains a native TS enum (Permission) which is a runtime value
        assert!(
            app_content.contains("import * as ex_shared from \"./ex_shared\""),
            "ex_app.ts should use `import * as` (not `import type * as`) for modules with native TS enums.\nContent:\n{app_content}"
        );
        assert!(
            !app_content.contains("import type * as ex_shared"),
            "ex_app.ts must NOT use `import type` for modules with native TS enums.\nContent:\n{app_content}"
        );

        // Print generated files for inspection
        println!("=== ex_shared.ts ===\n{shared_content}");
        println!("=== ex_app.ts ===\n{app_content}");
    }

    #[test]
    fn export_to_jsonschema_files() {
        let types = specta::collect_types();
        let constants = specta::collect_constants();

        let resolved = specta_serde::apply(types)
            .expect("serde transformation failed")
            .with_constants(constants);

        let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("output/jsonschema");

        specta_jsonschema::JsonSchema::default()
            .layout(specta_jsonschema::Layout::Files)
            .export_to(&out_dir, &resolved)
            .expect("jsonschema export failed");
        // single file output
        specta_jsonschema::JsonSchema::default()
            .layout(specta_jsonschema::Layout::SingleFile)
            .export_to(out_dir.join("singlefile.json"), &resolved)
            .expect("jsonschema export failed");

        // Layout::Files creates one .schema.json per type, organized by module
        let user_path = out_dir.join("ex_shared/User.schema.json");
        let pagination_path = out_dir.join("ex_shared/Pagination.schema.json");
        let app_config_path = out_dir.join("ex_app/AppConfig.schema.json");
        let user_list_path = out_dir.join("ex_app/UserListResponse.schema.json");

        assert!(user_path.exists(), "User.schema.json should be generated");
        assert!(
            pagination_path.exists(),
            "Pagination.schema.json should be generated"
        );
        assert!(
            app_config_path.exists(),
            "AppConfig.schema.json should be generated"
        );
        assert!(
            user_list_path.exists(),
            "UserListResponse.schema.json should be generated"
        );

        let user_content = fs::read_to_string(&user_path).unwrap();
        let app_config_content = fs::read_to_string(&app_config_path).unwrap();

        // Print sample generated files for inspection
        println!("=== User.schema.json ===\n{user_content}");
        println!("=== AppConfig.schema.json ===\n{app_config_content}");
    }

    #[test]
    fn export_to_zod_files() {
        let types = specta::collect_types();
        let constants = specta::collect_constants();

        let serde_resolved = specta_serde::apply(types.clone())
            .expect("serde transformation failed")
            .with_constants(constants.clone());
        let resolved = ResolvedTypes::from_types_and_constants(types, constants);
        let sr_str = format!("{serde_resolved:#?}");
        let r_str = format!("{resolved:#?}");

        let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("output/zod");

        specta_zod::Zod::default()
            .output_type_infers(false)
            .layout(specta_zod::Layout::Files)
            .export_to(&out_dir, &serde_resolved)
            .expect("zod export failed");

        // Read the generated files
        let shared_path = out_dir.join("ex_shared.ts");
        let app_path = out_dir.join("ex_app.ts");
        fs::write(out_dir.join("resolved.txt"), r_str).unwrap();
        fs::write(out_dir.join("serde_resolved.txt"), sr_str).unwrap();

        assert!(
            shared_path.exists(),
            "ex_shared.ts should be generated (zod)"
        );
        assert!(app_path.exists(), "ex_app.ts should be generated (zod)");

        let shared_content = fs::read_to_string(&shared_path).unwrap();
        let app_content = fs::read_to_string(&app_path).unwrap();

        // Zod schemas should contain z. schema definitions
        assert!(
            shared_content.contains("z."),
            "ex_shared.ts should contain Zod schemas"
        );
        assert!(
            app_content.contains("z."),
            "ex_app.ts should contain Zod schemas"
        );

        // Print generated files for inspection
        println!("=== ex_shared.ts (zod) ===\n{shared_content}");
        println!("=== ex_app.ts (zod) ===\n{app_content}");
    }
}
