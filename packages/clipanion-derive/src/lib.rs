extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod macros;
#[macro_use]
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
pub fn program(args: TokenStream, input: TokenStream) -> TokenStream {
    let input
        = parse_macro_input!(input as DeriveInput);

    match macros::program::program_macro(args, input) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn cli_enum(args: TokenStream, input: TokenStream) -> TokenStream {
    let input
        = parse_macro_input!(input as DeriveInput);

    match macros::cli_enum::cli_enum_macro(args, input) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn cli_exec_async(args: TokenStream, input: TokenStream) -> TokenStream {
    let input
        = parse_macro_input!(input as DeriveInput);

    match macros::cli_exec_async::cli_exec_async_macro(args, input) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn cli_exec_sync(args: TokenStream, input: TokenStream) -> TokenStream {
    let input
        = parse_macro_input!(input as DeriveInput);

    match macros::cli_exec_sync::cli_exec_sync_macro(args, input) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn cli_provider(args: TokenStream, input: TokenStream) -> TokenStream {
    let input
        = parse_macro_input!(input as DeriveInput);

    match macros::cli_provider::cli_provider_macro(args, input) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}
