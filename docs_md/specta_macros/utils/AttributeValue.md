# Enum `AttributeValue` -- path: `specta_macros::utils::AttributeValue`

```rust
pub enum AttributeValue {
	/// Literal value. Eg. `#[specta(name = "hello")]` or `#[specta(name = u32)]`
	Lit(syn::Lit),
	/// Path value. Eg. `#[specta(type = String)]` or `#[specta(type = ::std::string::String)]`
	/// Path doesn't follow the Rust spec hence the need for this custom parser. We are doing this anyway for backwards compatibility.
	Path(syn::Path),
	/// Expression value for values that are not valid paths.
	/// This allows us to later parse richer forms such as tuple/array/reference types.
	Expr(syn::Expr),
	/// A nested attribute. Eg. the `deprecated(note = "some note") in `#[specta(deprecated(note = "some note"))]`
	/// [Attribute](./Attribute.md)
	Attribute { span: proc_macro2::Span, attr: Vec<Attribute> },
}
```
