use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, ExprLit, Lit};

use crate::{shared::expect_lit, utils::AttributeBag};

pub fn program_macro(args: TokenStream, input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let mut command_attribute_bag
        = syn::parse::<AttributeBag>(args)?;

    let is_async = command_attribute_bag.take("async")
        .map(expect_lit!(Lit::Bool))
        .transpose()?
        .map(|lit| lit.value)
        .unwrap_or(false);

    if is_async {
        Ok(TokenStream::from(quote! {
            #[clipanion::derive::cli_enum]
            #[clipanion::derive::cli_exec_async]
            #[clipanion::derive::cli_provider]
            #input
        }))
    } else {
        Ok(TokenStream::from(quote! {
            #[clipanion::derive::cli_enum]
            #[clipanion::derive::cli_exec_sync]
            #[clipanion::derive::cli_provider]
            #input
        }))
    }
}
