extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput, Fields};

use crate::shared;

pub fn cli_enum_macro(_args: TokenStream, mut input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let syn::Data::Enum(enum_input) = &mut input.data else {
        panic!("Only enums are supported");
    };

    let enum_ident
        = &input.ident;
    let partial_enum_ident
        = shared::get_partial_enum_ident(&input.ident);

    let mut partial_enum_items
        = vec![];
    let mut from_match_arms
        = vec![];
    let mut trait_impls
        = vec![];

    for variant in &enum_input.variants {
        let variant_ident
            = &variant.ident;

        let Fields::Unnamed(fields) = &variant.fields else {
            panic!("Only unnamed fields are supported");
        };

        if fields.unnamed.len() != 1 {
            panic!("Only one field is supported");
        }

        let variant_ty
            = &fields.unnamed.first().unwrap().ty;

        let partial_ty: syn::Type
            = syn::parse_quote!{<#variant_ty as clipanion::details::CommandController>::Partial};
    
        {
            let mut partial_fields
                = Punctuated::new();

            partial_fields.push(syn::Field {
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                ident: None,
                colon_token: None,
                ty: partial_ty.clone(),
                mutability: syn::FieldMutability::None,
            });

            let partial_fields = syn::Fields::Unnamed(syn::FieldsUnnamed {
                paren_token: Default::default(),
                unnamed: partial_fields,
            });

            partial_enum_items.push(syn::Variant {
                ident: variant_ident.clone(),
                fields: partial_fields,
                attrs: vec![],
                discriminant: None,
            });
        }

        from_match_arms.push(quote!{
            #partial_enum_ident::#variant_ident(inner) => {
                #variant_ty::try_from(inner).map(Into::into)
            },
        });

        trait_impls.push(quote!{
            impl ::core::convert::From<#variant_ty> for #enum_ident {
                fn from(value: #variant_ty) -> Self {
                    Self::#variant_ident(value)
                }
            }

            impl ::core::convert::From<#partial_ty> for #partial_enum_ident {
                fn from(partial: #partial_ty) -> Self {
                    Self::#variant_ident(partial)
                }
            }

            impl ::core::convert::From<#enum_ident> for #variant_ty {
                fn from(value: #enum_ident) -> Self {
                    match value {
                        #enum_ident::#variant_ident(value) => value,
                        _ => unreachable!("expected {}, got something else", stringify!(#enum_ident)),
                    }
                }
            }
        });
    }

    Ok(TokenStream::from(quote! {
        #input

        #[derive(Debug)]
        pub enum #partial_enum_ident {
            #(#partial_enum_items,)*
        }

        impl ::clipanion::details::CliEnums for #enum_ident {
            type PartialEnum = #partial_enum_ident;
            type Enum = #enum_ident;
        }

        impl ::core::convert::TryFrom<#partial_enum_ident> for #enum_ident {
            type Error = ::clipanion::core::CommandError;

            fn try_from(value: #partial_enum_ident) -> Result<Self, ::clipanion::core::CommandError> {
                match value {
                    #(#from_match_arms)*
                }
            }
        }

        #(#trait_impls)*
    }))
}
