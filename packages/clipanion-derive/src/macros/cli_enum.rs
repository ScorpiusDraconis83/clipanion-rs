extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;

pub fn cli_enum_macro(types: Punctuated<syn::Path, syn::Token![,]>, mut enum_item: syn::ItemEnum) -> Result<TokenStream, syn::Error> {
    let enum_ident = &enum_item.ident;
    let enum_generics = &enum_item.generics;

    let mut trait_impls = Vec::new();

    for (i, ty) in types.iter().enumerate() {
        let variant_ident = format_ident!("_Variant{}", i + 1);
        let mut unnamed_fields = Punctuated::new();

        unnamed_fields.push(syn::Field {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            ident: None,
            colon_token: None,
            ty: syn::Type::Path(syn::TypePath { qself: None, path: ty.clone() }),
            mutability: syn::FieldMutability::None,
        });

        let fields = syn::Fields::Unnamed(syn::FieldsUnnamed {
            paren_token: Default::default(),
            unnamed: unnamed_fields,
        });

        enum_item.variants.push(syn::Variant {
            ident: variant_ident.clone(),
            fields,
            attrs: vec![],
            discriminant: None,
        });

        // From<T> for Enum
        trait_impls.push(quote! {
            impl #enum_generics From<#ty> for #enum_ident #enum_generics {
                fn from(value: #ty) -> Self {
                    #enum_ident::#variant_ident(value)
                }
            }
        });

        // From<Enum> for T
        trait_impls.push(quote! {
            impl #enum_generics ::core::convert::From<#enum_ident #enum_generics> for #ty {
                fn from(value: #enum_ident #enum_generics) -> Self {
                    match value {
                        #enum_ident::#variant_ident(inner) => inner,
                        _ => panic!(concat!("Cannot convert from ", stringify!(#enum_ident), " to ", stringify!(#ty))),
                    }
                }
            }
        });
    }

    let expanded = quote! {
        #enum_item

        #(#trait_impls)*
    };

    Ok(TokenStream::from(expanded))
}
