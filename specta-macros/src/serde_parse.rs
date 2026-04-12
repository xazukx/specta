//! Shared serde attribute parsing utilities.
//!
//! This module provides common types and helpers for parsing `#[serde(...)]` attributes,
//! used by both the `type` and `companion` macro implementations.

use inflector::Inflector;
use syn::{Attribute, LitStr, Meta, Result, Type, meta::ParseNestedMeta};

// ─── RenameRule ────────────────────────────────────────────────────────────────

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

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lower => "lowercase",
            Self::Upper => "UPPERCASE",
            Self::Pascal => "PascalCase",
            Self::Camel => "camelCase",
            Self::Snake => "snake_case",
            Self::ScreamingSnake => "SCREAMING_SNAKE_CASE",
            Self::Kebab => "kebab-case",
            Self::ScreamingKebab => "SCREAMING-KEBAB-CASE",
        }
    }
}

// ─── Attribute structs ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ConversionType {
    pub type_src: String,
    pub ty: Type,
}

#[derive(Default)]
pub struct ContainerAttrs {
    pub rename_serialize: Option<String>,
    pub rename_deserialize: Option<String>,
    pub rename_all_serialize: Option<RenameRule>,
    pub rename_all_deserialize: Option<RenameRule>,
    pub rename_all_fields_serialize: Option<RenameRule>,
    pub rename_all_fields_deserialize: Option<RenameRule>,
    pub tag: Option<String>,
    pub content: Option<String>,
    pub untagged: bool,
    pub default: bool,
    pub transparent: bool,
    pub from: Option<ConversionType>,
    pub try_from: Option<ConversionType>,
    pub into: Option<ConversionType>,
    pub variant_identifier: bool,
    pub field_identifier: bool,
}

#[derive(Default)]
pub struct VariantAttrs {
    pub rename_serialize: Option<String>,
    pub rename_deserialize: Option<String>,
    pub aliases: Vec<String>,
    pub rename_all_serialize: Option<RenameRule>,
    pub rename_all_deserialize: Option<RenameRule>,
    pub skip_serializing: bool,
    pub skip_deserializing: bool,
    pub has_serialize_with: bool,
    pub has_deserialize_with: bool,
    pub has_with: bool,
    pub other: bool,
    pub untagged: bool,
}

#[derive(Default)]
pub struct FieldAttrs {
    pub rename_serialize: Option<String>,
    pub rename_deserialize: Option<String>,
    pub aliases: Vec<String>,
    pub default: bool,
    pub flatten: bool,
    pub skip_serializing: bool,
    pub skip_deserializing: bool,
    pub skip_serializing_if: Option<String>,
    pub has_serialize_with: bool,
    pub has_deserialize_with: bool,
    pub has_with: bool,
}

// ─── Parsing helpers ───────────────────────────────────────────────────────────

/// Parse `= "literal"` from a meta item and return the `LitStr`.
pub fn parse_lit_str(meta: &ParseNestedMeta<'_>) -> Result<LitStr> {
    meta.value()?.parse()
}

/// Parse `= "literal"` from a meta item and return the string value.
pub fn parse_string_assignment(meta: &ParseNestedMeta<'_>) -> Result<String> {
    parse_lit_str(meta).map(|lit| lit.value())
}

/// Consume remaining value or nested group from an unrecognized meta attribute.
///
/// When `parse_nested_meta` encounters an unknown attribute like
/// `skip_serializing_if = "Option::is_none"`, the callback must consume the
/// `= "..."` portion. Otherwise the parser will report "expected ," because
/// the unconsumed tokens remain in the stream.
pub fn skip_meta_value(meta: &ParseNestedMeta<'_>) {
    if meta.input.peek(syn::Token![=]) {
        let _ = meta.value().and_then(|v| v.parse::<LitStr>());
    } else if meta.input.peek(syn::token::Paren) {
        let _ = meta.parse_nested_meta(|_| Ok(()));
    }
}

// ─── Attribute parsing ─────────────────────────────────────────────────────────

pub fn parse_container_attrs(attrs: &[Attribute]) -> Result<Option<ContainerAttrs>> {
    let mut parsed = ContainerAttrs::default();
    let mut found = false;

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        let Meta::List(list) = &attr.meta else {
            continue;
        };

        found = true;
        list.parse_nested_meta(|meta| parse_container_meta(&mut parsed, meta))?;
    }

    Ok(found.then_some(parsed))
}

pub fn parse_variant_attrs(attrs: &[Attribute]) -> Result<Option<VariantAttrs>> {
    let mut parsed = VariantAttrs::default();
    let mut found = false;

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        let Meta::List(list) = &attr.meta else {
            continue;
        };

        found = true;
        list.parse_nested_meta(|meta| parse_variant_meta(&mut parsed, meta))?;
    }

    Ok(found.then_some(parsed))
}

pub fn parse_field_attrs(attrs: &[Attribute]) -> Result<Option<FieldAttrs>> {
    let mut parsed = FieldAttrs::default();
    let mut found = false;

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        let Meta::List(list) = &attr.meta else {
            continue;
        };

        found = true;
        list.parse_nested_meta(|meta| parse_field_meta(&mut parsed, meta))?;
    }

    Ok(found.then_some(parsed))
}

fn parse_container_meta(target: &mut ContainerAttrs, meta: ParseNestedMeta<'_>) -> Result<()> {
    if meta.path.is_ident("rename") {
        parse_rename(
            &meta,
            &mut target.rename_serialize,
            &mut target.rename_deserialize,
        )?;
    } else if meta.path.is_ident("rename_all") {
        parse_rename_all(
            &meta,
            &mut target.rename_all_serialize,
            &mut target.rename_all_deserialize,
        )?;
    } else if meta.path.is_ident("rename_all_fields") {
        parse_rename_all(
            &meta,
            &mut target.rename_all_fields_serialize,
            &mut target.rename_all_fields_deserialize,
        )?;
    } else if meta.path.is_ident("tag") {
        target.tag = Some(parse_string_assignment(&meta)?);
    } else if meta.path.is_ident("content") {
        target.content = Some(parse_string_assignment(&meta)?);
    } else if meta.path.is_ident("untagged") {
        target.untagged = true;
    } else if meta.path.is_ident("default") {
        parse_default_assignment(&meta)?;
        target.default = true;
    } else if meta.path.is_ident("transparent") {
        target.transparent = true;
    } else if meta.path.is_ident("from") {
        target.from = Some(parse_conversion_assignment(&meta)?);
    } else if meta.path.is_ident("try_from") {
        target.try_from = Some(parse_conversion_assignment(&meta)?);
    } else if meta.path.is_ident("into") {
        target.into = Some(parse_conversion_assignment(&meta)?);
    } else if meta.path.is_ident("variant_identifier") {
        target.variant_identifier = true;
    } else if meta.path.is_ident("field_identifier") {
        target.field_identifier = true;
    } else {
        skip_meta_value(&meta);
    }

    Ok(())
}

pub fn parse_variant_meta(target: &mut VariantAttrs, meta: ParseNestedMeta<'_>) -> Result<()> {
    if meta.path.is_ident("rename") {
        parse_rename(
            &meta,
            &mut target.rename_serialize,
            &mut target.rename_deserialize,
        )?;
    } else if meta.path.is_ident("alias") {
        target.aliases.push(parse_string_assignment(&meta)?);
    } else if meta.path.is_ident("rename_all") {
        parse_rename_all(
            &meta,
            &mut target.rename_all_serialize,
            &mut target.rename_all_deserialize,
        )?;
    } else if meta.path.is_ident("skip") {
        target.skip_serializing = true;
        target.skip_deserializing = true;
    } else if meta.path.is_ident("skip_serializing") {
        target.skip_serializing = true;
    } else if meta.path.is_ident("skip_deserializing") {
        target.skip_deserializing = true;
    } else if meta.path.is_ident("serialize_with") {
        target.has_serialize_with = true;
        parse_string_assignment(&meta)?;
    } else if meta.path.is_ident("deserialize_with") {
        target.has_deserialize_with = true;
        parse_string_assignment(&meta)?;
    } else if meta.path.is_ident("with") {
        target.has_with = true;
        parse_string_assignment(&meta)?;
    } else if meta.path.is_ident("other") {
        target.other = true;
    } else if meta.path.is_ident("untagged") {
        target.untagged = true;
    } else {
        skip_meta_value(&meta);
    }

    Ok(())
}

pub fn parse_field_meta(target: &mut FieldAttrs, meta: ParseNestedMeta<'_>) -> Result<()> {
    if meta.path.is_ident("rename") {
        parse_rename(
            &meta,
            &mut target.rename_serialize,
            &mut target.rename_deserialize,
        )?;
    } else if meta.path.is_ident("alias") {
        target.aliases.push(parse_string_assignment(&meta)?);
    } else if meta.path.is_ident("default") {
        parse_default_assignment(&meta)?;
        target.default = true;
    } else if meta.path.is_ident("flatten") {
        target.flatten = true;
    } else if meta.path.is_ident("skip") {
        target.skip_serializing = true;
        target.skip_deserializing = true;
    } else if meta.path.is_ident("skip_serializing") {
        target.skip_serializing = true;
    } else if meta.path.is_ident("skip_deserializing") {
        target.skip_deserializing = true;
    } else if meta.path.is_ident("skip_serializing_if") {
        target.skip_serializing_if = Some(parse_string_assignment(&meta)?);
    } else if meta.path.is_ident("serialize_with") {
        target.has_serialize_with = true;
        parse_string_assignment(&meta)?;
    } else if meta.path.is_ident("deserialize_with") {
        target.has_deserialize_with = true;
        parse_string_assignment(&meta)?;
    } else if meta.path.is_ident("with") {
        target.has_with = true;
        parse_string_assignment(&meta)?;
    } else {
        skip_meta_value(&meta);
    }

    Ok(())
}

fn parse_rename(
    meta: &ParseNestedMeta<'_>,
    rename_serialize: &mut Option<String>,
    rename_deserialize: &mut Option<String>,
) -> Result<()> {
    if meta.input.peek(syn::Token![=]) {
        let value = parse_string_assignment(meta)?;
        *rename_serialize = Some(value.clone());
        *rename_deserialize = Some(value);
        return Ok(());
    }

    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("serialize") {
            *rename_serialize = Some(parse_string_assignment(&meta)?);
        } else if meta.path.is_ident("deserialize") {
            *rename_deserialize = Some(parse_string_assignment(&meta)?);
        }

        Ok(())
    })
}

fn parse_rename_all(
    meta: &ParseNestedMeta<'_>,
    rename_serialize: &mut Option<RenameRule>,
    rename_deserialize: &mut Option<RenameRule>,
) -> Result<()> {
    if meta.input.peek(syn::Token![=]) {
        let lit = parse_lit_str(meta)?;
        let rule = RenameRule::parse(&lit)?;
        *rename_serialize = Some(rule);
        *rename_deserialize = Some(rule);
        return Ok(());
    }

    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("serialize") {
            *rename_serialize = Some(RenameRule::parse(&parse_lit_str(&meta)?)?);
        } else if meta.path.is_ident("deserialize") {
            *rename_deserialize = Some(RenameRule::parse(&parse_lit_str(&meta)?)?);
        }

        Ok(())
    })
}

fn parse_default_assignment(meta: &ParseNestedMeta<'_>) -> Result<String> {
    if meta.input.peek(syn::Token![=]) {
        parse_string_assignment(meta)
    } else {
        Ok("__default__".to_owned())
    }
}

fn parse_conversion_assignment(meta: &ParseNestedMeta<'_>) -> Result<ConversionType> {
    let lit = parse_lit_str(meta)?;
    let type_src = lit.value();
    let ty = syn::parse_str::<Type>(&type_src)
        .map_err(|err| syn::Error::new(lit.span(), format!("invalid type literal: {err}")))?;

    Ok(ConversionType { type_src, ty })
}

// ─── Companion helpers ─────────────────────────────────────────────────────────

/// Resolve the serialized name for a field or variant.
///
/// Priority:
/// 1. Explicit `rename_serialize` on the field/variant
/// 2. `rename_all_serialize` on the container, applied to the Rust name
/// 3. The Rust name as-is
pub fn resolve_serialized_name(
    rust_name: &str,
    rename_serialize: Option<&str>,
    container_rename_all: Option<RenameRule>,
) -> String {
    if let Some(rename) = rename_serialize {
        rename.to_owned()
    } else if let Some(rule) = container_rename_all {
        rule.apply(rust_name)
    } else {
        rust_name.to_owned()
    }
}

/// Convert a string to PascalCase for use as an enum variant name.
pub fn to_pascal_case(s: &str) -> String {
    s.to_pascal_case()
}
