use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Result};

use super::AttributeScope;
use crate::serde_parse::{
    ContainerAttrs, ConversionType, FieldAttrs, RenameRule, VariantAttrs, parse_container_attrs,
    parse_field_attrs, parse_variant_attrs,
};

pub(super) fn lower_runtime_attributes(
    crate_ref: &TokenStream,
    scope: AttributeScope,
    raw_attrs: &[Attribute],
) -> Result<Option<TokenStream>> {
    match scope {
        AttributeScope::Container => parse_container_attrs(raw_attrs)
            .map(|attrs| attrs.map(|attrs| lower_container_attrs(crate_ref, attrs))),
        AttributeScope::Variant => parse_variant_attrs(raw_attrs)
            .map(|attrs| attrs.map(|attrs| lower_variant_attrs(crate_ref, attrs))),
        AttributeScope::Field => parse_field_attrs(raw_attrs)
            .map(|attrs| attrs.map(|attrs| lower_field_attrs(crate_ref, attrs))),
    }
}

fn lower_container_attrs(crate_ref: &TokenStream, attrs: ContainerAttrs) -> TokenStream {
    let mut inserts = Vec::new();
    push_opt_string(
        &mut inserts,
        "serde:container:rename_serialize",
        &attrs.rename_serialize,
    );
    push_opt_string(
        &mut inserts,
        "serde:container:rename_deserialize",
        &attrs.rename_deserialize,
    );
    push_opt_rename_rule(
        &mut inserts,
        "serde:container:rename_all_serialize",
        attrs.rename_all_serialize,
    );
    push_opt_rename_rule(
        &mut inserts,
        "serde:container:rename_all_deserialize",
        attrs.rename_all_deserialize,
    );
    push_opt_rename_rule(
        &mut inserts,
        "serde:container:rename_all_fields_serialize",
        attrs.rename_all_fields_serialize,
    );
    push_opt_rename_rule(
        &mut inserts,
        "serde:container:rename_all_fields_deserialize",
        attrs.rename_all_fields_deserialize,
    );
    push_opt_string(&mut inserts, "serde:container:tag", &attrs.tag);
    push_opt_string(&mut inserts, "serde:container:content", &attrs.content);
    push_bool(&mut inserts, "serde:container:untagged", attrs.untagged);
    push_bool(&mut inserts, "serde:container:default", attrs.default);
    push_bool(
        &mut inserts,
        "serde:container:transparent",
        attrs.transparent,
    );
    push_opt_conversion(&mut inserts, crate_ref, "from", &attrs.from);
    push_opt_conversion(&mut inserts, crate_ref, "try_from", &attrs.try_from);
    push_opt_conversion(&mut inserts, crate_ref, "into", &attrs.into);
    push_bool(
        &mut inserts,
        "serde:container:variant_identifier",
        attrs.variant_identifier,
    );
    push_bool(
        &mut inserts,
        "serde:container:field_identifier",
        attrs.field_identifier,
    );

    quote!(#(#inserts)*)
}

fn lower_variant_attrs(_crate_ref: &TokenStream, attrs: VariantAttrs) -> TokenStream {
    let mut inserts = Vec::new();
    push_opt_string(
        &mut inserts,
        "serde:variant:rename_serialize",
        &attrs.rename_serialize,
    );
    push_opt_string(
        &mut inserts,
        "serde:variant:rename_deserialize",
        &attrs.rename_deserialize,
    );
    push_vec_string(&mut inserts, "serde:variant:aliases", &attrs.aliases);
    push_opt_rename_rule(
        &mut inserts,
        "serde:variant:rename_all_serialize",
        attrs.rename_all_serialize,
    );
    push_opt_rename_rule(
        &mut inserts,
        "serde:variant:rename_all_deserialize",
        attrs.rename_all_deserialize,
    );
    push_bool(
        &mut inserts,
        "serde:variant:skip_serializing",
        attrs.skip_serializing,
    );
    push_bool(
        &mut inserts,
        "serde:variant:skip_deserializing",
        attrs.skip_deserializing,
    );
    push_bool(
        &mut inserts,
        "serde:variant:has_serialize_with",
        attrs.has_serialize_with,
    );
    push_bool(
        &mut inserts,
        "serde:variant:has_deserialize_with",
        attrs.has_deserialize_with,
    );
    push_bool(&mut inserts, "serde:variant:has_with", attrs.has_with);
    push_bool(&mut inserts, "serde:variant:other", attrs.other);
    push_bool(&mut inserts, "serde:variant:untagged", attrs.untagged);

    quote!(#(#inserts)*)
}

fn lower_field_attrs(_crate_ref: &TokenStream, attrs: FieldAttrs) -> TokenStream {
    let mut inserts = Vec::new();
    push_opt_string(
        &mut inserts,
        "serde:field:rename_serialize",
        &attrs.rename_serialize,
    );
    push_opt_string(
        &mut inserts,
        "serde:field:rename_deserialize",
        &attrs.rename_deserialize,
    );
    push_vec_string(&mut inserts, "serde:field:aliases", &attrs.aliases);
    push_bool(&mut inserts, "serde:field:default", attrs.default);
    push_bool(&mut inserts, "serde:field:flatten", attrs.flatten);
    push_bool(
        &mut inserts,
        "serde:field:skip_serializing",
        attrs.skip_serializing,
    );
    push_bool(
        &mut inserts,
        "serde:field:skip_deserializing",
        attrs.skip_deserializing,
    );
    push_opt_string(
        &mut inserts,
        "serde:field:skip_serializing_if",
        &attrs.skip_serializing_if,
    );
    push_bool(
        &mut inserts,
        "serde:field:has_serialize_with",
        attrs.has_serialize_with,
    );
    push_bool(
        &mut inserts,
        "serde:field:has_deserialize_with",
        attrs.has_deserialize_with,
    );
    push_bool(&mut inserts, "serde:field:has_with", attrs.has_with);

    quote!(#(#inserts)*)
}

fn push_opt_string(inserts: &mut Vec<TokenStream>, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        inserts.push(quote!(attrs.insert(#key, ::std::string::String::from(#value));));
    }
}

fn push_vec_string(inserts: &mut Vec<TokenStream>, key: &str, value: &[String]) {
    if value.is_empty() {
        return;
    }

    let value = value
        .iter()
        .map(|value| quote!(::std::string::String::from(#value)));
    inserts.push(quote!(attrs.insert(#key, vec![#(#value),*]);));
}

fn push_bool(inserts: &mut Vec<TokenStream>, key: &str, value: bool) {
    if value {
        inserts.push(quote!(attrs.insert(#key, true);));
    }
}

fn push_opt_rename_rule(inserts: &mut Vec<TokenStream>, key: &str, value: Option<RenameRule>) {
    if let Some(value) = value {
        let value = value.as_str();
        inserts.push(quote!(attrs.insert(#key, ::std::string::String::from(#value));));
    }
}

fn push_opt_conversion(
    inserts: &mut Vec<TokenStream>,
    crate_ref: &TokenStream,
    key: &str,
    value: &Option<ConversionType>,
) {
    if let Some(value) = value {
        let type_src = &value.type_src;
        let ty = &value.ty;
        let src_key = format!("serde:container:{key}_type_src");
        let resolved_key = format!("serde:container:{key}_resolved");
        inserts.push(quote!(attrs.insert(#src_key, ::std::string::String::from(#type_src));));
        inserts.push(
            quote!(attrs.insert(#resolved_key, <#ty as #crate_ref::Type>::definition(types));),
        );
    }
}
