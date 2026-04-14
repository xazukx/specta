# Struct `ExportPipeline` -- path: `specta::export::pipeline::ExportPipeline`

The main orchestrator for file-based export.

Handles module graph construction, filesystem layout, path resolution,
file writing, and stale file cleanup. Delegates language-specific
rendering to the [`ExportLanguage`] trait.

```rust
pub struct ExportPipeline {
}
```

## Methods

```rust
/**
`export_to` -- Export types to a directory using the given language exporter and layout.

For `Layout::FlatFile` and `Layout::ModulePrefixedName`, `path` is a file path.
For `Layout::Files`, `path` is a directory.
*/
pub fn export_to<L>(lang: &L, layout: &Layout, path: &Path, resolved: &ResolvedTypes) -> Result<(), L::Error>
/**
`export_to_string` -- Export types to a single string (for `FlatFile` and `ModulePrefixedName` layouts).
*/
pub fn export_to_string<L>(lang: &L, _layout: &Layout, resolved: &ResolvedTypes) -> Result<String, L::Error>
/**
`build_path_resolver` -- Build a PathResolver for the given types and layout without writing files.
*/
pub fn build_path_resolver<L>(lang: &L, layout: &Layout, types: &Types, constants: &Constants) -> PathResolver
```

