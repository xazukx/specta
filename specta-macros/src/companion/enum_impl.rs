use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, Ident};

use super::attr::CompanionFieldAttr;
use super::serde_rename::{
    SerdeContainerRename, parse_serde_item_rename, resolve_serialized_name,
};
use crate::utils::{parse_attrs, unraw_raw_ident};

pub fn generate_enum_companion(
    ident: &Ident,
    data: &DataEnum,
    serde_container: &SerdeContainerRename,
) -> syn::Result<TokenStream> {
    let mut variant_names = Vec::new();

    for variant in &data.variants {
        let mut attrs = parse_attrs(&variant.attrs)?;
        let companion_attr = CompanionFieldAttr::from_attrs(&mut attrs)?;
        let serde_item = parse_serde_item_rename(&variant.attrs)?;

        if companion_attr.skip || serde_item.skip {
            continue;
        }

        let rust_name = unraw_raw_ident(&variant.ident);
        let serialized_name = resolve_serialized_name(&rust_name, &serde_item, serde_container);
        variant_names.push(serialized_name);
    }

    Ok(quote! {
        impl #ident {
            /// Variant names as they would be serialized by serde.
            pub const VARIANT_NAMES: &'static [&'static str] = &[
                #(#variant_names,)*
            ];

            /// Returns the variant names as serialized by serde.
            pub fn variant_names() -> &'static [&'static str] {
                Self::VARIANT_NAMES
            }
        }
    })
}
