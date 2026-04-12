use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataStruct, Fields, GenericParam, Generics, Ident, Visibility};

use super::attr::{CompanionContainerAttr, CompanionFieldAttr};
use crate::serde_parse::{
    ContainerAttrs, parse_field_attrs, resolve_serialized_name, to_pascal_case,
};
use crate::utils::{parse_attrs, unraw_raw_ident};

/// Information about a single struct field for companion generation.
struct FieldInfo {
    /// The Rust field identifier.
    rust_ident: Ident,
    /// The Rust field type.
    ty: syn::Type,
    /// The serde-resolved serialized name.
    serialized_name: String,
    /// The PascalCase variant name for the Field/Value enums.
    variant_name: Ident,
    /// The PascalCase variant name as a string.
    variant_name_str: String,
    /// The Rust type as a string for CompanionField::type_str().
    type_str: String,
    /// Optional title from `#[companion(title = "...")]`.
    title: Option<String>,
    /// Optional order from `#[companion(order = N)]`.
    order: Option<isize>,
}

/// Whether the struct has any generic type parameters.
fn has_type_generics(generics: &Generics) -> bool {
    generics
        .params
        .iter()
        .any(|p| matches!(p, GenericParam::Type(_)))
}

pub fn generate_struct_companion(
    vis: &Visibility,
    ident: &Ident,
    generics: &Generics,
    data: &DataStruct,
    container_attr: &CompanionContainerAttr,
    serde_container: &ContainerAttrs,
    crate_ref: &TokenStream,
) -> syn::Result<TokenStream> {
    let fields = match &data.fields {
        Fields::Named(named) => &named.named,
        Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(
                ident,
                "TypeCompanion: tuple structs are not supported. Only named-field structs are allowed.",
            ));
        }
        Fields::Unit => {
            return Err(syn::Error::new_spanned(
                ident,
                "TypeCompanion: unit structs are not supported. Only named-field structs are allowed.",
            ));
        }
    };

    // Collect field info, skipping fields marked with #[companion(skip)] or #[serde(skip)].
    let mut field_infos = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let mut attrs = parse_attrs(&field.attrs)?;
        let companion_attr = CompanionFieldAttr::from_attrs(&mut attrs)?;
        let serde_field = parse_field_attrs(&field.attrs)?.unwrap_or_default();

        if companion_attr.skip || serde_field.skip_serializing || serde_field.skip_deserializing {
            continue;
        }

        let rust_name = unraw_raw_ident(field_ident);
        let serialized_name = resolve_serialized_name(
            &rust_name,
            serde_field.rename_serialize.as_deref(),
            serde_container.rename_all_serialize,
        );
        let variant_str = to_pascal_case(&serialized_name);
        let variant_name = format_ident!("{}", variant_str, span = field_ident.span());
        field_infos.push(FieldInfo {
            rust_ident: field_ident.clone(),
            ty: field.ty.clone(),
            serialized_name,
            variant_name,
            variant_name_str: variant_str,
            type_str: type_str_from_type(&field.ty),
            title: companion_attr.title,
            order: companion_attr.order,
        });
    }

    let field_enum_ident = format_ident!("{}Field", ident);
    let value_enum_ident = format_ident!("{}Value", ident);

    let is_generic = has_type_generics(generics);

    let field_enum = generate_field_enum(
        vis,
        &field_enum_ident,
        &field_infos,
        container_attr,
        crate_ref,
    );
    let value_enum = generate_value_enum(
        vis,
        &value_enum_ident,
        &field_enum_ident,
        &field_infos,
        container_attr,
        crate_ref,
        generics,
        is_generic,
    );
    let struct_impl = generate_struct_impl(
        ident,
        &field_enum_ident,
        &value_enum_ident,
        &field_infos,
        container_attr,
        crate_ref,
        generics,
    );

    Ok(quote! {
        #field_enum
        #value_enum
        #struct_impl
    })
}

fn type_str_from_type(ty: &syn::Type) -> String {
    quote!(#ty).to_string()
}

fn generate_field_enum(
    vis: &Visibility,
    enum_ident: &Ident,
    fields: &[FieldInfo],
    container_attr: &CompanionContainerAttr,
    crate_ref: &TokenStream,
) -> TokenStream {
    let variants: Vec<_> = fields.iter().map(|f| &f.variant_name).collect();

    let extra_derives = &container_attr.derive_field;
    let serde_attrs: Vec<_> = container_attr
        .serde_field
        .iter()
        .map(|s| quote!(#s))
        .collect();
    let serde_attr_line = if serde_attrs.is_empty() {
        quote!()
    } else {
        quote!(#[serde(#(#serde_attrs),*)])
    };

    // Display impl: returns the serialized field name
    let display_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let name = &f.serialized_name;
            quote!(Self::#variant => write!(f, #name))
        })
        .collect();

    // FromStr impl: parses from serialized name and PascalCase variant name
    let fromstr_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let name = &f.serialized_name;
            let variant_name = &f.variant_name_str;
            if name != variant_name {
                quote!(#name | #variant_name => Ok(Self::#variant))
            } else {
                quote!(#name => Ok(Self::#variant))
            }
        })
        .collect();

    // CompanionField impl
    let name_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let name = &f.serialized_name;
            quote!(Self::#variant => #name)
        })
        .collect();

    let type_str_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let ts = &f.type_str;
            quote!(Self::#variant => #ts)
        })
        .collect();

    let title_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let title = f.title.as_deref().unwrap_or(&f.serialized_name);
            quote!(Self::#variant => #title)
        })
        .collect();

    let order_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let order = f.order.unwrap_or(0);
            quote!(Self::#variant => #order)
        })
        .collect();

    let fields_count = fields.len();

    quote! {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, #(#extra_derives),*)]
        #serde_attr_line
        #vis enum #enum_ident {
            #(#variants,)*
        }

        #[automatically_derived]
        impl ::core::fmt::Display for #enum_ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(#display_arms,)*
                }
            }
        }

        #[automatically_derived]
        impl ::core::str::FromStr for #enum_ident {
            type Err = ::std::string::String;

            fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                match s {
                    #(#fromstr_arms,)*
                    _ => Err(::std::format!("unknown field: {}", s)),
                }
            }
        }

        #[automatically_derived]
        impl #crate_ref::companion::CompanionField for #enum_ident {
            fn name(&self) -> &'static str {
                match self {
                    #(#name_arms,)*
                }
            }

            fn type_str(&self) -> &'static str {
                match self {
                    #(#type_str_arms,)*
                }
            }

            fn title(&self) -> &'static str {
                match self {
                    #(#title_arms,)*
                }
            }

            fn order(&self) -> isize {
                match self {
                    #(#order_arms,)*
                }
            }
        }

        impl #enum_ident {
            /// All field variants as a static array.
            pub const FIELDS: &'static [#enum_ident; #fields_count] = &[
                #(#enum_ident::#variants,)*
            ];
        }
    }
}

fn generate_value_enum(
    vis: &Visibility,
    enum_ident: &Ident,
    field_enum_ident: &Ident,
    fields: &[FieldInfo],
    container_attr: &CompanionContainerAttr,
    crate_ref: &TokenStream,
    generics: &Generics,
    is_generic: bool,
) -> TokenStream {
    let variants: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let ty = &f.ty;
            quote!(#variant(#ty))
        })
        .collect();

    let extra_derives = &container_attr.derive_value;
    let serde_attrs: Vec<_> = container_attr
        .serde_value
        .iter()
        .map(|s| quote!(#s))
        .collect();
    let serde_attr_line = if serde_attrs.is_empty() {
        quote!()
    } else {
        quote!(#[serde(#(#serde_attrs),*)])
    };

    // Generic parameters for the value enum
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    // CompanionValue impl
    let field_name_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let name = &f.serialized_name;
            quote!(Self::#variant(_) => #name)
        })
        .collect();

    let type_name_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let ts = &f.type_str;
            quote!(Self::#variant(_) => #ts)
        })
        .collect();

    // TryFrom impls — only for non-generic structs
    let try_from_impls = if is_generic {
        quote!()
    } else {
        generate_try_from_impls(enum_ident, field_enum_ident, fields)
    };

    // field() match arms
    let field_match_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            quote!(Self::#variant(_) => #field_enum_ident::#variant)
        })
        .collect();

    quote! {
        #[derive(Clone, Debug, PartialEq, #(#extra_derives),*)]
        #serde_attr_line
        #vis enum #enum_ident #impl_generics #where_clause {
            #(#variants,)*
        }

        #[automatically_derived]
        impl #impl_generics #enum_ident #type_generics #where_clause {
            /// Returns the field variant this value corresponds to.
            pub fn field(&self) -> #field_enum_ident {
                match self {
                    #(#field_match_arms,)*
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics #crate_ref::companion::CompanionValue for #enum_ident #type_generics #where_clause {
            fn field_name(&self) -> &'static str {
                match self {
                    #(#field_name_arms,)*
                }
            }

            fn type_name(&self) -> &'static str {
                match self {
                    #(#type_name_arms,)*
                }
            }
        }

        #try_from_impls
    }
}

fn generate_try_from_impls(
    value_enum_ident: &Ident,
    field_enum_ident: &Ident,
    fields: &[FieldInfo],
) -> TokenStream {
    let mut seen_types = std::collections::HashSet::new();
    let mut impls = Vec::new();

    for field in fields {
        let type_key = &field.type_str;
        if seen_types.contains(type_key) {
            continue;
        }
        seen_types.insert(type_key.clone());

        // Skip generic/complex types to avoid orphan rule issues
        if type_key.contains('<') || type_key.contains("impl ") || type_key.contains("dyn ") {
            continue;
        }

        let ty = &field.ty;
        let variant = &field.variant_name;

        // Collect all variants with the same type
        let matching_variants: Vec<_> = fields.iter().filter(|f| f.type_str == *type_key).collect();

        // Only generate TryFrom if there's exactly one variant with this type
        // (otherwise the conversion is ambiguous)
        if matching_variants.len() == 1 {
            // TryFrom<Value> for T
            impls.push(quote! {
                #[automatically_derived]
                impl ::core::convert::TryFrom<#value_enum_ident> for #ty {
                    type Error = #value_enum_ident;

                    fn try_from(value: #value_enum_ident) -> ::core::result::Result<Self, Self::Error> {
                        match value {
                            #value_enum_ident::#variant(inner) => Ok(inner),
                            other => Err(other),
                        }
                    }
                }
            });

            // TryFrom<(Field, T)> for Value
            impls.push(quote! {
                #[automatically_derived]
                impl ::core::convert::TryFrom<(#field_enum_ident, #ty)> for #value_enum_ident {
                    type Error = (# field_enum_ident, #ty);

                    fn try_from((field, value): (#field_enum_ident, #ty)) -> ::core::result::Result<Self, Self::Error> {
                        match field {
                            #field_enum_ident::#variant => Ok(#value_enum_ident::#variant(value)),
                            _ => Err((field, value)),
                        }
                    }
                }
            });
        }
    }

    quote!(#(#impls)*)
}

fn generate_struct_impl(
    struct_ident: &Ident,
    field_enum_ident: &Ident,
    value_enum_ident: &Ident,
    fields: &[FieldInfo],
    container_attr: &CompanionContainerAttr,
    crate_ref: &TokenStream,
    generics: &Generics,
) -> TokenStream {
    let field_names: Vec<_> = fields.iter().map(|f| &f.serialized_name).collect();

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let value_fn_name = format_ident!("{}", container_attr.value_fn.as_deref().unwrap_or("value"));
    let update_fn_name = format_ident!(
        "{}",
        container_attr.update_fn.as_deref().unwrap_or("update")
    );
    let fields_fn_name = format_ident!(
        "{}",
        container_attr.fields_fn.as_deref().unwrap_or("fields")
    );

    let value_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let rust_ident = &f.rust_ident;
            quote!(#field_enum_ident::#variant => #value_enum_ident::#variant(self.#rust_ident.clone()))
        })
        .collect();

    let update_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let rust_ident = &f.rust_ident;
            quote!(#value_enum_ident::#variant(v) => self.#rust_ident = v)
        })
        .collect();

    let as_values_items: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = &f.variant_name;
            let rust_ident = &f.rust_ident;
            quote!(#value_enum_ident::#variant(self.#rust_ident.clone()))
        })
        .collect();

    quote! {
        impl #impl_generics #struct_ident #type_generics #where_clause {
            /// The serde-serialized field names.
            pub const FIELD_NAMES: &'static [&'static str] = &[
                #(#field_names,)*
            ];

            /// Returns all field enum variants.
            pub fn #fields_fn_name() -> &'static [#field_enum_ident] {
                #field_enum_ident::FIELDS
            }

            /// Returns the field names as serialized by serde.
            pub fn field_names() -> &'static [&'static str] {
                Self::FIELD_NAMES
            }

            /// Returns the value of a specific field.
            pub fn #value_fn_name(&self, field: #field_enum_ident) -> #value_enum_ident #type_generics {
                match field {
                    #(#value_arms,)*
                }
            }

            /// Updates the value of a specific field.
            pub fn #update_fn_name(&mut self, value: #value_enum_ident #type_generics) {
                match value {
                    #(#update_arms,)*
                }
            }

            /// Returns all field values as a vector.
            pub fn as_values(&self) -> Vec<#value_enum_ident #type_generics> {
                vec![
                    #(#as_values_items,)*
                ]
            }
        }

        #[automatically_derived]
        impl #impl_generics #crate_ref::companion::TypeCompanion<#field_enum_ident, #value_enum_ident #type_generics> for #struct_ident #type_generics #where_clause {
            fn value(&self, field: #field_enum_ident) -> #value_enum_ident #type_generics {
                self.#value_fn_name(field)
            }

            fn update(&mut self, value: #value_enum_ident #type_generics) {
                self.#update_fn_name(value);
            }

            fn fields() -> &'static [#field_enum_ident] {
                #field_enum_ident::FIELDS
            }

            fn as_values(&self) -> Vec<#value_enum_ident #type_generics> {
                self.as_values()
            }
        }
    }
}
