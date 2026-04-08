use proc_macro2::TokenStream;
use syn::Result;

use crate::utils::{AttrExtract, Attribute, AttributeValue};

/// Container-level `#[companion(...)]` attributes.
#[derive(Default)]
pub struct CompanionContainerAttr {
    /// Extra derives to add to the Field enum.
    pub derive_field: Vec<TokenStream>,
    /// Extra derives to add to the Value enum.
    pub derive_value: Vec<TokenStream>,
    /// Serde attributes to apply to the Field enum.
    pub serde_field: Vec<TokenStream>,
    /// Serde attributes to apply to the Value enum.
    pub serde_value: Vec<TokenStream>,
    /// Rename the `.value()` method.
    pub value_fn: Option<String>,
    /// Rename the `.update()` method.
    pub update_fn: Option<String>,
    /// Rename the `.fields()` method.
    pub fields_fn: Option<String>,
    /// Skip auto-collection (when `collect` feature is on).
    pub collect: Option<bool>,
}

impl CompanionContainerAttr {
    pub fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut result = Self::default();

        for attr in attrs.extract_all("companion", "derive_field") {
            match &attr.value {
                Some(AttributeValue::Attribute {
                    attr: nested_attrs, ..
                }) => {
                    for nested in nested_attrs {
                        let path = nested.key.to_string();
                        result
                            .derive_field
                            .push(syn::parse_str::<TokenStream>(&path)?);
                    }
                }
                Some(AttributeValue::Path(path)) => {
                    result.derive_field.push(quote::quote!(#path));
                }
                _ => {
                    return Err(syn::Error::new(
                        attr.value_span(),
                        "companion: expected derive_field(Trait, ...)",
                    ));
                }
            }
        }

        for attr in attrs.extract_all("companion", "derive_value") {
            match &attr.value {
                Some(AttributeValue::Attribute {
                    attr: nested_attrs, ..
                }) => {
                    for nested in nested_attrs {
                        let path = nested.key.to_string();
                        result
                            .derive_value
                            .push(syn::parse_str::<TokenStream>(&path)?);
                    }
                }
                Some(AttributeValue::Path(path)) => {
                    result.derive_value.push(quote::quote!(#path));
                }
                _ => {
                    return Err(syn::Error::new(
                        attr.value_span(),
                        "companion: expected derive_value(Trait, ...)",
                    ));
                }
            }
        }

        for attr in attrs.extract_all("companion", "serde_field") {
            match &attr.value {
                Some(AttributeValue::Attribute {
                    attr: nested_attrs, ..
                }) => {
                    for nested in nested_attrs {
                        let key = &nested.key;
                        if let Some(val) = &nested.value {
                            let val_ts: TokenStream = match val {
                                AttributeValue::Lit(lit) => quote::quote!(#lit),
                                AttributeValue::Path(path) => quote::quote!(#path),
                                AttributeValue::Expr(expr) => quote::quote!(#expr),
                                _ => {
                                    return Err(syn::Error::new(
                                        nested.value_span(),
                                        "companion: unsupported serde_field attribute value",
                                    ));
                                }
                            };
                            result.serde_field.push(quote::quote!(#key = #val_ts));
                        } else {
                            result.serde_field.push(quote::quote!(#key));
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        attr.value_span(),
                        "companion: expected serde_field(...)",
                    ));
                }
            }
        }

        for attr in attrs.extract_all("companion", "serde_value") {
            match &attr.value {
                Some(AttributeValue::Attribute {
                    attr: nested_attrs, ..
                }) => {
                    for nested in nested_attrs {
                        let key = &nested.key;
                        if let Some(val) = &nested.value {
                            let val_ts: TokenStream = match val {
                                AttributeValue::Lit(lit) => quote::quote!(#lit),
                                AttributeValue::Path(path) => quote::quote!(#path),
                                AttributeValue::Expr(expr) => quote::quote!(#expr),
                                _ => {
                                    return Err(syn::Error::new(
                                        nested.value_span(),
                                        "companion: unsupported serde_value attribute value",
                                    ));
                                }
                            };
                            result.serde_value.push(quote::quote!(#key = #val_ts));
                        } else {
                            result.serde_value.push(quote::quote!(#key));
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        attr.value_span(),
                        "companion: expected serde_value(...)",
                    ));
                }
            }
        }

        if let Some(attr) = attrs.extract("companion", "value_fn") {
            result.value_fn = Some(attr.parse_string()?);
        }

        if let Some(attr) = attrs.extract("companion", "update_fn") {
            result.update_fn = Some(attr.parse_string()?);
        }

        if let Some(attr) = attrs.extract("companion", "fields_fn") {
            result.fields_fn = Some(attr.parse_string()?);
        }

        if let Some(attr) = attrs.extract("companion", "collect") {
            result.collect = Some(attr.parse_bool()?);
        }

        Ok(result)
    }
}

/// Field-level `#[companion(...)]` attributes.
#[derive(Default)]
pub struct CompanionFieldAttr {
    /// Exclude this field/variant entirely.
    pub skip: bool,
    /// Human-readable title.
    pub title: Option<String>,
    /// Ordering hint.
    pub order: Option<isize>,
}

impl CompanionFieldAttr {
    pub fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut result = Self::default();

        if let Some(attr) = attrs.extract("companion", "skip") {
            result.skip = attr.parse_bool().unwrap_or(true);
        }

        if let Some(attr) = attrs.extract("companion", "title") {
            result.title = Some(attr.parse_string()?);
        }

        if let Some(attr) = attrs.extract("companion", "order") {
            match &attr.value {
                Some(AttributeValue::Lit(syn::Lit::Int(lit))) => {
                    result.order = Some(lit.base10_parse::<isize>()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        attr.value_span(),
                        "companion: expected integer literal for order",
                    ));
                }
            }
        }

        Ok(result)
    }
}
