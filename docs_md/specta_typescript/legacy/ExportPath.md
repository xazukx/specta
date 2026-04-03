# Struct `ExportPath` -- path: `specta_typescript::legacy::ExportPath`

Represents the path of an error in the export tree.
This is designed to be opaque, meaning it's internal format and `Display` impl are subject to change at will.

```rust
pub struct ExportPath (
	String,
)
```
