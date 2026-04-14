# Enum `ImportStyle` -- path: `specta::export::layout::ImportStyle`

Controls how cross-module imports are structured.

```rust
pub enum ImportStyle {
	/// Namespace/wildcard imports: `import * as alias from "./path";`
	Namespace,
	/// Named imports: `import { TypeA, TypeB } from "./path";`
	/// References use the type name directly instead of `alias.TypeName`.
	Named,
}
```
