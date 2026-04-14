# Functions in specta::export::path_resolver

```rust
/**
`module_alias` -- Compute the module alias for namespace imports.
E.g., `"shared::item"` -> `"shared$item"`
*/
pub fn module_alias(module_path: &str) -> String
/**
`relative_import_path` -- Compute a relative import path between two module paths.

This is the core path resolution logic shared by all TypeScript-family exporters.
*/
pub fn relative_import_path(from_module_path: &str, to_module_path: &str) -> String
```

