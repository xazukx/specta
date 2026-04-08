//! Easily export your Rust types to other languages
//!
//! This crate contains the macro which are reexported by the `specta` crate.
//! You shouldn't need to use this crate directly.
//! Checkout [Specta](https://docs.rs/specta).
#![doc(
    html_logo_url = "https://github.com/specta-rs/specta/raw/main/.github/logo-128.png",
    html_favicon_url = "https://github.com/specta-rs/specta/raw/main/.github/logo-128.png"
)]

#[cfg(feature = "companion")]
mod companion;
mod constant;
#[cfg(feature = "DO_NOT_USE_function")]
mod specta;
mod r#type;
mod utils;

use quote::quote;
use syn::{Error, LitStr, Type, parse_macro_input};

/// Implements `specta::Type` for a given struct or enum.
///
/// # Attributes
/// Attributes can be applied to modify Specta's behavior. Specta can natively read `#[serde(...)]` attributes so your generally recommend to [just use them](https://serde.rs/attributes.html).
///
/// Specta also introduces some of it's own attributes:
///  - `#[specta(optional)]` - When paired with an `Option<T>` field, this will result in `{ a?: T | null }` instead of `{ a: T | null }`.
///  - `#[specta(type = ::std::string::String)]` - Will override the type of a item, variant or field to a given type.
///  - `#[specta(collect = false)]` - When using the `collect` feature, this will prevent the specific type from being exported.
///
/// ## Example
///
/// ```ignore
/// use specta::Type;
///
/// // Use it on structs
/// #[derive(Type)]
/// pub struct MyCustomStruct {
///     pub name: String,
/// }
///
/// #[derive(Type)]
/// pub struct MyCustomStruct2(String, i32, bool);
///
/// // Use it on enums
/// #[derive(Type)]
/// pub enum MyCustomType {
///     VariantOne,
///     VariantTwo(String, i32),
///     VariantThree { name: String, age: i32 },
/// }
/// ```
#[proc_macro_derive(Type, attributes(specta))]
pub fn derive_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    r#type::derive(input).unwrap_or_else(|err| err.into_compile_error().into())
}

/// Marks a `const` item for TypeScript export.
///
/// This attribute macro wraps a bare `const` item, generating a hidden struct
/// that implements `specta::Constant` alongside the original const.
///
/// # Attributes
///
/// - `name = "TS_NAME"` - Override the TypeScript export name. Default: the Rust const name.
/// - `output = "string"` or `output = "bytes"` - Control how the value is serialized.
///
/// ## Example
///
/// ```ignore
/// #[specta::specta_const]
/// pub const MAX_RETRIES: u32 = 5;
/// // Outputs: export const MAX_RETRIES = 5 as const;
///
/// #[specta::specta_const(name = "MAX_RETRY_COUNT")]
/// pub const MAX_RETRIES: u32 = 5;
/// // Outputs: export const MAX_RETRY_COUNT = 5 as const;
/// ```
#[proc_macro_attribute]
pub fn specta_const(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    constant::attribute(attr, item).unwrap_or_else(|err| err.into_compile_error().into())
}

/// Parses a string literal into a Rust type token stream.
///
/// This is an internal helper proc macro used by Specta macros to turn a
/// literal like `"Option<String>"` into a Rust type at compile time.
#[proc_macro]
pub fn parse_type_from_lit(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let lit = parse_macro_input!(input as LitStr);

    match syn::parse_str::<Type>(&lit.value()) {
        Ok(ty) => quote!(#ty).into(),
        Err(err) => Error::new_spanned(lit, format!("invalid type literal: {err}"))
            .to_compile_error()
            .into(),
    }
}

/// Generates companion enums for structs (field + value enums) and variant name
/// arrays for enums, enabling dynamic field access and updates at runtime.
///
/// ## On Structs
///
/// Generates `{Struct}Field` and `{Struct}Value` enums, a `FIELD_NAMES` constant,
/// and `value()`, `update()`, `fields()`, `as_values()` methods. Also implements
/// the `TypeCompanion` trait.
///
/// ## On Enums
///
/// Generates a `VARIANT_NAMES` constant and `variant_names()` method.
///
/// ## Attributes
///
/// Container-level: `#[companion(derive_field(Trait), derive_value(Trait), value_fn = "name", ...)]`
/// Field-level: `#[companion(skip, title = "Title", order = N)]`
///
/// Serde rename attributes (`#[serde(rename = "...")]`, `#[serde(rename_all = "...")]`)
/// are respected for computing serialized names.
#[proc_macro_derive(TypeCompanion, attributes(companion))]
#[cfg(feature = "companion")]
pub fn derive_type_companion(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    companion::derive(input).unwrap_or_else(|err| err.into_compile_error().into())
}

/// Prepares a function to have its types extracted using `specta::function::fn_datatype!`
///
/// ## Example
///
/// ```ignore
/// #[specta::specta]
/// fn my_function(arg1: i32, arg2: bool) -> &'static str {
///     "Hello World"
/// }
/// ```
#[proc_macro_attribute]
#[cfg(feature = "DO_NOT_USE_function")]
pub fn specta(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    specta::attribute(item).unwrap_or_else(|err| err.into_compile_error().into())
}
