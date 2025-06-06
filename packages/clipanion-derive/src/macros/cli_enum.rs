extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

use crate::shared;

pub fn cli_enum_macro(types: Punctuated<syn::Path, syn::Token![,]>, item: syn::ItemEnum) -> Result<TokenStream, syn::Error> {
    let item_ident
        = &item.ident;

    let (partial_enum_ident, enum_ident)
        = shared::get_cli_enum_names(&item_ident);

    let mut partial_enum_items
        = vec![];

    let mut enum_items
        = vec![];

    let mut from_match_arms
        = vec![];

    let mut trait_impls
        = vec![];

    for (i, ty) in types.iter().enumerate() {
        let variant_ident
            = shared::get_command_variant_ident(i, ty);

        let partial_ty: syn::Type
            = syn::parse_quote!{<#ty as clipanion::details::CommandController>::Partial};
    
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

        {
            let mut fields
                = Punctuated::new();

            fields.push(syn::Field {
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                ident: None,
                colon_token: None,
                ty: syn::Type::Path(syn::TypePath {qself: None, path: ty.clone()}),
                mutability: syn::FieldMutability::None,
            });

            let fields = syn::Fields::Unnamed(syn::FieldsUnnamed {
                paren_token: Default::default(),
                unnamed: fields,
            });

            enum_items.push(syn::Variant {
                ident: variant_ident.clone(),
                fields,
                attrs: vec![],
                discriminant: None,
            });
        }

        from_match_arms.push(quote!{
            #partial_enum_ident::#variant_ident(inner) => {
                #ty::try_from(inner).map(Into::into)
            },
        });

        trait_impls.push(quote!{
            impl ::core::convert::From<#ty> for #enum_ident {
                fn from(value: #ty) -> Self {
                    Self::#variant_ident(value)
                }
            }

            impl ::core::convert::From<#partial_ty> for #partial_enum_ident {
                fn from(partial: #partial_ty) -> Self {
                    Self::#variant_ident(partial)
                }
            }

            impl ::core::convert::From<#enum_ident> for #ty {
                fn from(value: #enum_ident) -> Self {
                    match value {
                        #enum_ident::#variant_ident(value) => {
                            value
                        },

                        _ => {
                            unreachable!("expected {}, got something else", stringify!(#enum_ident))
                        },
                    }
                }
            }
        });
    }

    Ok(TokenStream::from(quote! {
        #item

        pub enum #partial_enum_ident {
            #(#partial_enum_items,)*
        }

        pub enum #enum_ident {
            #(#enum_items,)*
        }

        impl ::clipanion::details::CliEnums for #item_ident {
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
