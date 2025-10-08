extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn cli_exec_async_macro(_args: TokenStream, mut input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let syn::Data::Enum(enum_input) = &mut input.data else {
        panic!("Only enums are supported");
    };

    let enum_ident
        = &input.ident;

    let mut match_arms
        = vec![];

    for variant in &enum_input.variants {
        let variant_ident
            = &variant.ident;

        match_arms.push(quote! {
            Self::#variant_ident(command) => command.execute().await.into(),
        });
    }

    let expanded = quote! {
        #input

        impl ::clipanion::details::CommandExecutorAsync for #enum_ident {
            async fn execute(self, env: &::clipanion::advanced::Environment) -> ::clipanion::details::CommandResult {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    Ok(TokenStream::from(expanded))
}
