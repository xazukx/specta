use inflector::Inflector;
use syn::{Attribute, LitStr, Meta, Result};

/// Supported serde rename rules.
#[derive(Copy, Clone)]
pub enum RenameRule {
    Lower,
    Upper,
    Pascal,
    Camel,
    Snake,
    ScreamingSnake,
    Kebab,
    ScreamingKebab,
}

impl RenameRule {
    pub fn parse(lit: &LitStr) -> Result<Self> {
        Ok(match lit.value().as_str() {
            "lowercase" => Self::Lower,
            "UPPERCASE" => Self::Upper,
            "PascalCase" => Self::Pascal,
            "camelCase" => Self::Camel,
            "snake_case" => Self::Snake,
            "SCREAMING_SNAKE_CASE" => Self::ScreamingSnake,
            "kebab-case" => Self::Kebab,
            "SCREAMING-KEBAB-CASE" => Self::ScreamingKebab,
            _ => {
                return Err(syn::Error::new(
                    lit.span(),
                    format!("unsupported serde casing: `{}`", lit.value()),
                ));
            }
        })
    }

    /// Apply this rename rule to a Rust identifier name.
    pub fn apply(self, name: &str) -> String {
        match self {
            Self::Lower => name.to_lowercase(),
            Self::Upper => name.to_uppercase(),
            Self::Pascal => name.to_pascal_case(),
            Self::Camel => name.to_camel_case(),
            Self::Snake => name.to_snake_case(),
            Self::ScreamingSnake => name.to_screaming_snake_case(),
            Self::Kebab => name.to_kebab_case(),
            Self::ScreamingKebab => {
                let kebab = name.to_kebab_case();
                kebab.to_uppercase()
            }
        }
    }
}

/// Serde attributes extracted from a container (`#[serde(...)]` on the struct/enum).
#[derive(Default)]
pub struct SerdeContainerRename {
    /// `#[serde(rename_all = "...")]` — applies to field/variant names.
    pub rename_all: Option<RenameRule>,
}

/// Serde attributes extracted from a field/variant (`#[serde(...)]`).
#[derive(Default)]
pub struct SerdeItemRename {
    /// `#[serde(rename = "...")]` — explicit rename.
    pub rename: Option<String>,
    /// `#[serde(skip)]` — skip this item.
    pub skip: bool,
}

/// Parse container-level serde rename attributes.
pub fn parse_serde_container_rename(attrs: &[Attribute]) -> Result<SerdeContainerRename> {
    let mut result = SerdeContainerRename::default();

    for attr in attrs.iter().filter(|a| a.path().is_ident("serde")) {
        let Meta::List(list) = &attr.meta else {
            continue;
        };

        list.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                if meta.input.peek(syn::Token![=]) {
                    let lit: LitStr = meta.value()?.parse()?;
                    result.rename_all = Some(RenameRule::parse(&lit)?);
                }
                // ignore rename_all(serialize = ..., deserialize = ...) — we use serialize
            }
            Ok(())
        })?;
    }

    Ok(result)
}

/// Parse field/variant-level serde rename attributes.
pub fn parse_serde_item_rename(attrs: &[Attribute]) -> Result<SerdeItemRename> {
    let mut result = SerdeItemRename::default();

    for attr in attrs.iter().filter(|a| a.path().is_ident("serde")) {
        let Meta::List(list) = &attr.meta else {
            continue;
        };

        list.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                if meta.input.peek(syn::Token![=]) {
                    let lit: LitStr = meta.value()?.parse()?;
                    result.rename = Some(lit.value());
                } else {
                    // rename(serialize = "...", deserialize = "...")
                    meta.parse_nested_meta(|inner| {
                        if inner.path.is_ident("serialize") {
                            let lit: LitStr = inner.value()?.parse()?;
                            result.rename = Some(lit.value());
                        }
                        // skip deserialize, we use the serialize name
                        Ok(())
                    })?;
                }
            } else if meta.path.is_ident("skip") {
                result.skip = true;
            } else if meta.path.is_ident("skip_serializing") {
                result.skip = true;
            } else if meta.path.is_ident("skip_deserializing") {
                result.skip = true;
            }
            Ok(())
        })?;
    }

    Ok(result)
}

/// Resolve the serialized name for a field or variant.
///
/// Priority:
/// 1. `#[serde(rename = "...")]` on the field/variant
/// 2. `#[serde(rename_all = "...")]` on the container, applied to the Rust name
/// 3. The Rust name as-is
pub fn resolve_serialized_name(
    rust_name: &str,
    item_rename: &SerdeItemRename,
    container_rename: &SerdeContainerRename,
) -> String {
    if let Some(ref rename) = item_rename.rename {
        rename.clone()
    } else if let Some(rule) = container_rename.rename_all {
        rule.apply(rust_name)
    } else {
        rust_name.to_owned()
    }
}

/// Convert a string to PascalCase for use as an enum variant name.
pub fn to_pascal_case(s: &str) -> String {
    // Handle various input formats by converting to snake_case first, then to PascalCase.
    // This handles: camelCase, kebab-case, SCREAMING_SNAKE_CASE, etc.
    s.to_pascal_case()
}

