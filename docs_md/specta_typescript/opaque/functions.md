# Functions in specta_typescript::opaque

```rust
/**
`define` -- Define a custom Typescript string which can be used as a `DataType::Reference`.

This is an advanced feature which should be used with caution.
*/
pub fn define<impl Into<Cow<'static, str>>>(raw: impl ) -> specta::datatype::Reference
```

