extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, punctuated::Punctuated, DeriveInput};

mod macros;
mod shared;
mod utils;

#[proc_macro_attribute]
pub fn command(args: TokenStream, input: TokenStream) -> TokenStream {
    let input
        = parse_macro_input!(input as DeriveInput);

    match macros::command::command_macro(args, input) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn cli_enum(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let types: Punctuated<syn::Path, syn::Token![,]>
        = parse_macro_input!(attrs with Punctuated::parse_terminated);

    let enum_item: syn::ItemEnum
        = parse_macro_input!(item as syn::ItemEnum);

    match macros::cli_enum::cli_enum_macro(types, enum_item) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn cli_exec_async(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let types: Punctuated<syn::Path, syn::Token![,]>
        = parse_macro_input!(attrs with Punctuated::parse_terminated);

    let enum_item: syn::ItemEnum
        = parse_macro_input!(item as syn::ItemEnum);

    match macros::cli_exec_async::cli_exec_async_macro(types, enum_item) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn cli_exec_sync(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let types: Punctuated<syn::Path, syn::Token![,]>
        = parse_macro_input!(attrs with Punctuated::parse_terminated);

    let enum_item: syn::ItemEnum
        = parse_macro_input!(item as syn::ItemEnum);

    match macros::cli_exec_sync::cli_exec_sync_macro(types, enum_item) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}
