# Enum `Fields` -- path: `specta::datatype::fields::Fields`

Data stored within an enum variant or struct.

```rust
pub enum Fields {
	/// Unit struct.
	/// 
	/// Represented in Rust as `pub struct Unit;` and in TypeScript as `null`.
	Unit,
	/// Struct with unnamed fields.
	/// 
	/// Represented in Rust as `pub struct Unit();` and in TypeScript as `[]`.
	/// [UnnamedFields](./UnnamedFields.md)
	Unnamed(UnnamedFields),
	/// Struct with named fields.
	/// 
	/// Represented in Rust as `pub struct Unit {}` and in TypeScript as `{}`.
	/// [NamedFields](./NamedFields.md)
	Named(NamedFields),
}
```
