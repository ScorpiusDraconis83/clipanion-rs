extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

use crate::shared;

pub fn cli_exec_async_macro(types: Punctuated<syn::Path, syn::Token![,]>, enum_item: syn::ItemEnum) -> Result<TokenStream, syn::Error> {
    let (_, enum_ident)
        = shared::get_cli_enum_names(&enum_item.ident);

    let mut match_arms
        = vec![];

    for (i, ty) in types.iter().enumerate() {
        let variant_ident
            = shared::get_command_variant_ident(i, ty);

        match_arms.push(quote! {
            Self::#variant_ident(command) => command.execute().await.into(),
        });
    }

    let expanded = quote! {
        #enum_item

        impl ::clipanion::details::CommandExecutorAsync for #enum_ident {
            async fn execute(&self, env: &::clipanion::advanced::Environment) -> ::clipanion::details::CommandResult {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    Ok(TokenStream::from(expanded))
}
