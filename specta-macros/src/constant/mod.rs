mod attr;

use attr::ConstantOutput;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ItemConst;

/// `#[specta_const]` attribute macro for bare `const` items.
///
/// Generates a hidden struct + `Constant` impl alongside the original const.
pub fn attribute(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> syn::Result<proc_macro::TokenStream> {
    let item_const: ItemConst = syn::parse(item)?;

    let const_ident = &item_const.ident;
    let const_name = const_ident.to_string();

    // Parse attribute arguments: #[specta_const(name = "...", output = "...", crate = ...)]
    let mut attrs = if attr.is_empty() {
        Vec::new()
    } else {
        let parsed: syn::punctuated::Punctuated<syn::Meta, syn::Token![,]> =
            syn::parse::Parser::parse(syn::punctuated::Punctuated::parse_terminated, attr)?;

        let mut result = Vec::new();
        for meta in parsed {
            match meta {
                syn::Meta::NameValue(nv) => {
                    let key = nv
                        .path
                        .get_ident()
                        .ok_or_else(|| syn::Error::new_spanned(&nv.path, "expected identifier"))?
                        .clone();
                    let value = match &nv.value {
                        syn::Expr::Lit(lit) => crate::utils::AttributeValue::Lit(lit.lit.clone()),
                        other => crate::utils::AttributeValue::Expr(other.clone()),
                    };
                    result.push(crate::utils::Attribute {
                        source: "specta_const".to_string(),
                        key,
                        value: Some(value),
                    });
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "specta_const: expected `key = value` arguments",
                    ));
                }
            }
        }
        result
    };

    // Extract name override
    let export_name = if let Some(pos) = attrs.iter().position(|a| a.key == "name") {
        let attr = attrs.swap_remove(pos);
        attr.parse_string()?
    } else {
        const_name.clone()
    };

    // Extract output format
    let output = if let Some(pos) = attrs.iter().position(|a| a.key == "output") {
        let attr = attrs.swap_remove(pos);
        let output_str = attr.parse_string()?;
        match output_str.as_str() {
            "string" => ConstantOutput::String,
            "bytes" => ConstantOutput::Bytes,
            other => {
                return Err(syn::Error::new(
                    attr.value_span(),
                    format!(
                        "specta_const: unknown output format \"{}\". Expected \"string\" or \"bytes\"",
                        other
                    ),
                ));
            }
        }
    } else {
        ConstantOutput::Infer
    };

    // Extract crate override
    let crate_ref = if let Some(pos) = attrs.iter().position(|a| a.key == "crate") {
        use quote::ToTokens;
        let attr = attrs.swap_remove(pos);
        attr.parse_path()?.to_token_stream()
    } else {
        quote!(specta)
    };

    // Check for unknown attributes
    if let Some(attr) = attrs.first() {
        return Err(syn::Error::new(
            attr.key.span(),
            format!("specta_const: unknown attribute '{}'", attr.key),
        ));
    }

    let hidden_struct = format_ident!("__specta_const_{}", const_ident);
    let value_expr = generate_value_expr(&crate_ref, const_ident, &item_const.expr, output)?;

    // Extract doc comments and deprecated from the const item
    let doc_lines: Vec<String> = item_const
        .attrs
        .iter()
        .filter_map(|a| {
            if a.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &a.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        return Some(s.value());
                    }
                }
            }
            None
        })
        .collect();
    let comments = doc_lines.join("\n");

    let is_deprecated = item_const
        .attrs
        .iter()
        .any(|a| a.path().is_ident("deprecated"));
    let deprecated_tokens = if is_deprecated {
        // Simple deprecated without details for attribute macro
        quote!({ Some(datatype::Deprecated::new()) })
    } else {
        quote!(None)
    };

    Ok(quote! {
        #item_const

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub struct #hidden_struct;

        const _: () = {
            use std::borrow::Cow;
            use #crate_ref::datatype;

            #[automatically_derived]
            impl #crate_ref::Constant for #hidden_struct {
                fn name() -> Cow<'static, str> {
                    Cow::Borrowed(#export_name)
                }

                fn value() -> datatype::ConstantValue {
                    #value_expr
                }

                fn docs() -> Cow<'static, str> {
                    Cow::Borrowed(#comments)
                }

                fn deprecated() -> Option<datatype::Deprecated> {
                    #deprecated_tokens
                }

                fn module_path() -> Cow<'static, str> {
                    Cow::Borrowed(module_path!())
                }
            }
        };
    }
    .into())
}

/// Generate the value expression with optional output format conversion.
///
/// Tries AST-based literal extraction first (handles wrapper types, byte strings, etc.).
/// Falls back to the trait-based approach when no literal can be extracted from the AST.
fn generate_value_expr(
    crate_ref: &TokenStream,
    const_ident: &syn::Ident,
    const_expr: &syn::Expr,
    output: ConstantOutput,
) -> syn::Result<TokenStream> {
    // Try AST-based extraction first
    if let Some(lit) = extract_literal(const_expr) {
        return literal_to_value_tokens(lit, output);
    }

    // Fall back to trait-based approach (e.g. for array expressions like [0xFF, 0xD8, 0xFF])
    let raw_value = quote!(#const_ident);
    Ok(match output {
        ConstantOutput::Infer => {
            quote!(#crate_ref::internal::IntoConstantValue::into_constant_value(&#raw_value))
        }
        ConstantOutput::String => {
            quote!(#crate_ref::internal::IntoConstantValueAsString::into_constant_value_as_string(&#raw_value))
        }
        ConstantOutput::Bytes => {
            quote!(#crate_ref::internal::IntoConstantValueAsBytes::into_constant_value_as_bytes(&#raw_value))
        }
    })
}

/// Recursively extract a literal from an expression tree.
/// Descends into call arguments (for wrapper types like `ItemId(TinyText("value"))`),
/// references, and parenthesized expressions.
fn extract_literal(expr: &syn::Expr) -> Option<&syn::Lit> {
    match expr {
        syn::Expr::Lit(lit) => Some(&lit.lit),
        syn::Expr::Call(call) => {
            for arg in &call.args {
                if let Some(lit) = extract_literal(arg) {
                    return Some(lit);
                }
            }
            None
        }
        syn::Expr::Reference(r) => extract_literal(&r.expr),
        syn::Expr::Paren(p) => extract_literal(&p.expr),
        _ => None,
    }
}

/// Convert a `syn::Lit` to `ConstantValue` construction tokens.
///
/// The generated tokens assume `datatype` and `Cow` are in scope (which they are
/// inside the generated `const _: () = { ... }` block).
fn literal_to_value_tokens(lit: &syn::Lit, output: ConstantOutput) -> syn::Result<TokenStream> {
    match lit {
        syn::Lit::Str(s) => {
            let val = s.value();
            match output {
                ConstantOutput::Bytes => {
                    let bytes = val.into_bytes();
                    Ok(quote!(datatype::ConstantValue::Bytes(Cow::Owned(
                        vec![#(#bytes),*]
                    ))))
                }
                _ => Ok(quote!(datatype::ConstantValue::String(Cow::Borrowed(#val)))),
            }
        }
        syn::Lit::ByteStr(bs) => match output {
            ConstantOutput::String => {
                let s = String::from_utf8_lossy(&bs.value()).into_owned();
                Ok(quote!(datatype::ConstantValue::String(Cow::Borrowed(#s))))
            }
            _ => {
                let bytes = bs.value();
                Ok(quote!(datatype::ConstantValue::Bytes(Cow::Owned(
                    vec![#(#bytes),*]
                ))))
            }
        },
        syn::Lit::Int(i) => {
            let n: u128 = i
                .base10_parse()
                .map_err(|e| syn::Error::new_spanned(i, format!("specta_const: {}", e)))?;
            Ok(quote!(datatype::ConstantValue::UnsignedInteger(#n)))
        }
        syn::Lit::Float(f) => {
            let n: f64 = f
                .base10_parse()
                .map_err(|e| syn::Error::new_spanned(f, format!("specta_const: {}", e)))?;
            Ok(quote!(datatype::ConstantValue::Float(datatype::FloatBits::from_f64(#n))))
        }
        syn::Lit::Bool(b) => {
            let val = b.value;
            Ok(quote!(datatype::ConstantValue::Bool(#val)))
        }
        other => Err(syn::Error::new_spanned(
            other,
            "specta_const: unsupported literal type",
        )),
    }
}

/// Convert a PascalCase or camelCase name to SCREAMING_SNAKE_CASE.
fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if !result.is_empty() {
                // Don't add underscore if previous char was already uppercase
                // and next is uppercase too (e.g. "HTTPStatus" -> "HTTP_STATUS")
                let last_was_upper = result.ends_with(|c: char| c.is_uppercase());
                let next_is_lower = chars.peek().is_some_and(|c| c.is_lowercase());
                if !last_was_upper || next_is_lower {
                    result.push('_');
                }
            }
            result.push(c.to_uppercase().next().unwrap());
        } else {
            result.push(c.to_uppercase().next().unwrap());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::to_screaming_snake_case;

    #[test]
    fn test_screaming_snake_case() {
        assert_eq!(to_screaming_snake_case("ApiVersion"), "API_VERSION");
        assert_eq!(to_screaming_snake_case("MaxRetries"), "MAX_RETRIES");
        assert_eq!(to_screaming_snake_case("HTTPStatus"), "HTTP_STATUS");
        assert_eq!(to_screaming_snake_case("Timeout"), "TIMEOUT");
        assert_eq!(to_screaming_snake_case("A"), "A");
        assert_eq!(to_screaming_snake_case("AB"), "AB");
        assert_eq!(to_screaming_snake_case("DefaultTimeout"), "DEFAULT_TIMEOUT");
    }
}
