extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Attribute, DeriveInput, LitStr, Meta, Token};

macro_rules! matches_pattern {
    ($pattern:pat) => {
        |val| match val {
            $pattern => true,
            _ => false,
        }
    };
}

macro_rules! extract_match {
    ($expression:path) => {
        |val| match val {
            $expression(value) => Some(value),
            _ => None,
        }
    };
}

#[derive(Debug)]
enum CommandAttribute {
    Default,
    Path(Vec<String>),
}

#[derive(Debug)]
enum OptionAttribute {
    Description(String),
    Option(Vec<String>),
    Positional,
    Proxy,
    Required,
}

fn extract_str_vec(attr: &Attribute) -> Result<Vec<String>, syn::Error> {
    if let Meta::List(meta_list) = &attr.meta {
        let punctuated = meta_list.parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)?;
        let values = punctuated.iter().map(|lit| lit.value()).collect::<Vec<_>>();

        Ok(values)
    } else {
        Err(syn::Error::new_spanned(attr, format!("Expected a list of string literals")))
    }
}

fn extract_str(attr: &Attribute) -> Result<String, syn::Error> {
    if let Meta::List(meta_list) = &attr.meta {
        let lit = meta_list.parse_args::<LitStr>()?;
        let value = lit.value();

        Ok(value)
    } else {
        Err(syn::Error::new_spanned(attr, format!("Expected a string literal")))
    }
}

fn extract_comma_separated_str(attr: &Attribute) -> Result<Vec<String>, syn::Error> {
    if let Meta::List(meta_list) = &attr.meta {
        let lit = meta_list.parse_args::<LitStr>()?;
        let value = lit.value();
        let values = value.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();

        Ok(values)
    } else {
        Err(syn::Error::new_spanned(attr, format!("Expected a list of string literals")))
    }
}

fn extract_command_attribute(attr: &Attribute) -> Result<Option<CommandAttribute>, syn::Error> {
    let path = attr.path();
    if path.segments.len() != 2 || path.segments[0].ident != "cli" {
        return Ok(None);
    }

    let name = path.segments[1].ident.to_string();
    match name.as_str() {
        "default" => Ok(Some(CommandAttribute::Default)),
        "path" => Ok(Some(CommandAttribute::Path(extract_str_vec(attr)?))),

        _ => Err(syn::Error::new_spanned(attr, format!("Unknown command attribute 'cli::{}'", name))),
    }
}

fn extract_option_attribute(attr: &Attribute) -> Result<Option<OptionAttribute>, syn::Error> {
    let path = attr.path();
    if path.segments.len() != 2 || path.segments[0].ident != "cli" {
        return Ok(None);
    }

    let name = path.segments[1].ident.to_string();

    match name.as_str() {
        "description" => Ok(Some(OptionAttribute::Description(extract_str(attr)?))),
        "option" => Ok(Some(OptionAttribute::Option(extract_comma_separated_str(attr)?))),
        "positional" => Ok(Some(OptionAttribute::Positional)),
        "proxy" => Ok(Some(OptionAttribute::Proxy)),
        "required" => Ok(Some(OptionAttribute::Required)),

        _ => Err(syn::Error::new_spanned(attr, format!("Unknown option attribute 'cli::{}'", name))),
    }
}

fn command_impl(mut input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let struct_input = if let syn::Data::Struct(data) = &mut input.data {
        data
    } else {
        panic!("Only structs are supported");
    };

    let struct_name = &input.ident;

    let mut command_attributes = vec![];
    let mut remaining_attributes = vec![];

    for attr in input.attrs.iter() {
        if let Some(command_attribute) = extract_command_attribute(&attr)? {
            command_attributes.push(command_attribute);
        } else {
            remaining_attributes.push(attr.clone());
        }
    }

    input.attrs = remaining_attributes;

    let mut builder = vec![];
    let mut has_path = false;

    for command_attribute in &command_attributes {
        match command_attribute {
            CommandAttribute::Default => {
                builder.push(quote! {
                    builder.make_default();
                });

                has_path = true;
            },

            CommandAttribute::Path(path) => {
                let path_literals = path.iter()
                    .map(|s| LitStr::new(s, proc_macro2::Span::call_site()))
                    .collect::<Vec<_>>();

                builder.push(quote! {
                    builder.add_path(vec![#(#path_literals.to_string()),*]);
                });

                has_path = true;
            },
        }
    }

    if !has_path {
        return Err(syn::Error::new_spanned(input, "The command must have a path"));
    }

    for field in &mut struct_input.fields {
        let mut option_attributes = vec![];
        let mut remaining_attributes = vec![];

        for attr in field.attrs.iter() {
            if let Some(option_attribute) = extract_option_attribute(&attr)? {
                option_attributes.push(option_attribute);
            } else {
                remaining_attributes.push(attr.clone());
            }
        }

        field.attrs = remaining_attributes;

        let mut field_type = &field.ty;
        let mut is_option_type = false;

        if let syn::Type::Path(type_path) = &field_type {
            if &type_path.path.segments[0].ident == "Option" {
                let inner_type = &type_path.path.segments[0].arguments;
                if let syn::PathArguments::AngleBracketed(args) = inner_type {
                    if let syn::GenericArgument::Type(ty) = &args.args[0] {
                        field_type = ty;
                        is_option_type = true;
                    }
                }
            }
        }

        for option_attribute in &option_attributes {
            match option_attribute {
                OptionAttribute::Option(name_set) => {
                    let mut arity = 1;

                    if let syn::Type::Path(type_path) = &field_type {
                        if &type_path.path.segments[0].ident == "bool" {
                            arity = 0;
                        }
                    }
            
                    if let syn::Type::Tuple(tuple) = field_type {
                        arity = tuple.elems.len();
                    }

                    let description = option_attributes.iter()
                        .find_map(extract_match!(OptionAttribute::Description))
                        .cloned()
                        .unwrap_or_default();

                    let is_required = option_attributes.iter()
                        .any(matches_pattern!(OptionAttribute::Required));
            
                    let name_set_literals = name_set.iter()
                        .map(|s| LitStr::new(s, proc_macro2::Span::call_site()))
                        .collect::<Vec<_>>();

                    builder.push(quote! {
                        builder.add_option(clipanion::core::OptionDefinition {
                            name_set: vec![#(#name_set_literals.to_string()),*],
                            description: #description.to_string(),
                            required: #is_required,
                            arity: #arity,
                            ..Default::default()
                        })?;
                    });
                },

                OptionAttribute::Positional => {
                    let field_name_upper = field.ident.as_ref().unwrap()
                        .to_string()
                        .to_uppercase();

                    let mut is_rest = false;

                    if let syn::Type::Path(type_path) = &field_type {
                        if &type_path.path.segments[0].ident == "Vec" {
                            is_rest = true;
                        }
                    }

                    let is_proxy = option_attributes.iter()
                        .any(matches_pattern!(OptionAttribute::Proxy));

                    if is_proxy {
                        builder.push(quote! {
                            builder.add_proxy(#field_name_upper);
                        });
                    } else if is_rest {
                        builder.push(quote! {
                            builder.add_rest(#field_name_upper);
                        });
                    } else {
                        builder.push(quote! {
                            builder.add_positional(!#is_option_type, #field_name_upper);
                        });
                    }
                },

                _ => {},
            }
        }
    }

    let expanded = quote! {
        #[derive(Default)]
        #input

        impl clipanion::advanced::CommandController for #struct_name {
            fn hydrate_cli_from(&mut self, state: clipanion::core::RunState) {
            }

            fn compile(builder: &mut clipanion::core::CommandBuilder) -> Result<clipanion::core::Machine, clipanion::core::BuildError> {
                let mut machine = clipanion::core::Machine::new();
                #(#builder)*
                Ok(machine)
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

#[proc_macro_attribute]
pub fn command(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match command_impl(input) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}
