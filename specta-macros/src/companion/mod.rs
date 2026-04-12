mod attr;
mod enum_impl;
mod struct_impl;

use quote::quote;
use syn::{Data, DeriveInput, parse};

use crate::serde_parse::parse_container_attrs;
use crate::utils::parse_attrs;

use self::attr::CompanionContainerAttr;
use self::enum_impl::generate_enum_companion;
use self::struct_impl::generate_struct_companion;

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<proc_macro::TokenStream> {
    let DeriveInput {
        ident,
        generics,
        data,
        attrs,
        vis,
    } = &parse::<DeriveInput>(input)?;

    let raw_attrs = attrs;
    let mut attrs = parse_attrs(raw_attrs)?;

    let container_attr = CompanionContainerAttr::from_attrs(&mut attrs)?;
    let serde_container = parse_container_attrs(raw_attrs)?.unwrap_or_default();

    let crate_ref = quote!(specta);

    // Check for unknown companion attributes
    if let Some(attr) = attrs.iter().find(|attr| attr.source == "companion") {
        if let Some(crate::utils::AttributeValue::Attribute {
            attr: inner_attrs, ..
        }) = &attr.value
        {
            if let Some(inner) = inner_attrs.first() {
                return Err(syn::Error::new(
                    inner.key.span(),
                    format!("companion: unknown attribute '{}'", inner.key),
                ));
            }
        }
    }

    let output = match data {
        Data::Struct(data) => generate_struct_companion(
            vis,
            ident,
            generics,
            data,
            &container_attr,
            &serde_container,
            &crate_ref,
        )?,
        Data::Enum(data) => generate_enum_companion(ident, data, &serde_container)?,
        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "TypeCompanion: union types are not supported.",
            ));
        }
    };

    Ok(output.into())
}
