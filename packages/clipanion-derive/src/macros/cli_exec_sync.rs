extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;

pub fn cli_exec_sync_macro(types: Punctuated<syn::Path, syn::Token![,]>, enum_item: syn::ItemEnum) -> Result<TokenStream, syn::Error> {
    let enum_ident = &enum_item.ident;
    let enum_generics = &enum_item.generics;

    let mut match_arms = Vec::new();

    for i in 0..types.len() {
        let variant_ident = format_ident!("_Variant{}", i + 1);

        match_arms.push(quote! {
            Self::#variant_ident(command) => command.execute().into(),
        });
    }

    let expanded = quote! {
        #enum_item

        impl #enum_generics ::clipanion::details::CommandExecutor for #enum_ident #enum_generics {
            fn execute(&self, env: &::clipanion::advanced::Environment) -> ::clipanion::details::CommandResult {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    Ok(TokenStream::from(expanded))
}
