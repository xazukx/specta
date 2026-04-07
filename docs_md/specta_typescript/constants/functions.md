# Functions in specta_typescript::constants

```rust
/**
`export_constant` -- Render a single constant as a TypeScript `export const` declaration.
*/
pub fn export_constant(exporter: &crate::Exporter, constant: &specta::NamedConstant) -> Result<String, crate::Error>
/**
`export_constants` -- Render a collection of constants as TypeScript declarations.
*/
pub fn export_constants(exporter: &crate::Exporter, constants: &specta::Constants) -> Result<String, crate::Error>
```

