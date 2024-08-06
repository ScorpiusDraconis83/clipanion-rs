extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, punctuated::Punctuated, Attribute, DeriveInput, Ident, Lit, LitBool, LitStr, Meta, Path, Token};

macro_rules! expect_lit {
    ($expression:path) => {
        |val| match val {
            $expression(value) => Ok(value),
            _ => Err(syn::Error::new_spanned(val, "Invalid literal type")),
        }
    };
}

fn to_lit_str<T: AsRef<str>>(str: T) -> LitStr {
    LitStr::new(str.as_ref(), proc_macro2::Span::call_site())
}

#[derive(Clone, Default)]
struct AttributeBag {
    attributes: HashMap<String, Lit>,
}

impl AttributeBag {
    pub fn expect_empty(&self) -> syn::Result<()> {
        if !self.attributes.is_empty() {
            return Err(syn::Error::new_spanned(self.attributes.iter().next().unwrap().1, "Unsupported extra attributes"));
        }

        Ok(())
    }

    pub fn take(&mut self, key: &str) -> Option<Lit> {
        self.attributes.remove(key)
    }
}

impl Parse for AttributeBag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Prepare a vector to hold the named parameters
        let mut attributes = HashMap::new();
        
        // Parse the remaining named parameters
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let name = ident.to_string();

            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;  // Consume the equals sign

                let value: Lit = input.parse()?;
                attributes.insert(name, value);
            } else {
                attributes.insert(name, Lit::Bool(LitBool::new(true, ident.span())));
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if !input.is_empty() {
            return Err(input.error("Unexpected token"));
        }

        Ok(Self {attributes})
    }
}

#[derive(Clone, Default)]
struct OptionBag {
    path: Vec<String>,
    attributes: AttributeBag,
}

impl OptionBag {
    fn parse_with_path(input: ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;

        let path = path.value()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();

        let mut attributes = AttributeBag::default();
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;            
            attributes = input.parse()?;
        }

        Ok(Self {
            path,
            attributes,
        })
    }

    fn parse_without_path(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: vec![],
            attributes: input.parse()?,
        })
    }
}

impl Parse for OptionBag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) {
            Self::parse_without_path(input)
        } else {
            Self::parse_with_path(input)
        }
    }
}

#[derive(Clone, Default)]
struct CliAttributes {
    attributes: HashMap<String, Vec<Attribute>>,
}

impl CliAttributes {
    fn parse_args<T: Default + Parse>(attr: &Attribute) -> syn::Result<T> {
        match attr.meta {
            Meta::Path(_) => Ok(T::default()),
            _ => attr.parse_args::<T>(),
        }
    }

    fn extract(attrs: &mut Vec<Attribute>) -> syn::Result<Self> {
        let mut cli_attributes = CliAttributes::default();
        let mut remaining_attributes = vec![];
    
        for attr in std::mem::take(attrs).into_iter(){
            let path = attr.path();
            if path.segments.len() == 0 || path.segments[0].ident != "cli" {
                remaining_attributes.push(attr.clone());
                continue;
            }
    
            if path.segments.len() != 2 {
                return Err(syn::Error::new_spanned(attr, "Expected a named attribute"));
            }
    
            let name = path.segments[1].ident.to_string();
    
            cli_attributes.attributes.entry(name)
                .or_insert_with(Vec::new)
                .push(attr);
        }

        *attrs = remaining_attributes;
    
        Ok(cli_attributes)
    }

    fn take_unique<T: Default + Parse>(&mut self, key: &str) -> syn::Result<Option<T>> {
        match self.attributes.remove(key) {
            Some(values) => {
                if values.len() > 1 {
                    return Err(syn::Error::new(proc_macro2::Span::call_site(), "Attribute must be unique"));
                }

                let attr = &values[0];
                Self::parse_args(attr).map(Some)
            },

            None => Ok(None),
        }
    }

    fn take_paths(&mut self) -> syn::Result<Vec<Vec<LitStr>>> {
        let path_attributes = self.attributes.remove("path")
            .unwrap_or_default();

        let punctuated_paths = path_attributes.into_iter()
            .map(|attr| attr.parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated))
            .collect::<syn::Result<Vec<_>>>()?;

        let path_lits = punctuated_paths.into_iter()
            .map(|punctuated| punctuated.into_iter().collect())
            .collect();

        Ok(path_lits)        
    }
}

fn command_impl(args: TokenStream, mut input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let struct_input = if let syn::Data::Struct(data) = &mut input.data {
        data
    } else {
        panic!("Only structs are supported");
    };

    let mut builder = vec![];

    let mut default_hydrater = vec![];
    let mut option_hydrater = vec![];
    let mut positional_hydrater = vec![];
    
    let mut command_cli_attributes
        = CliAttributes::extract(&mut input.attrs)?;

    let mut command_attribute_bag
        = syn::parse::<AttributeBag>(args)?;

    let is_default = command_attribute_bag.take("default")
        .map(expect_lit!(Lit::Bool))
        .transpose()?
        .map(|lit| lit.value)
        .unwrap_or(false);

    let is_proxy = command_attribute_bag.take("proxy")
        .map(expect_lit!(Lit::Bool))
        .transpose()?
        .map(|lit| lit.value)
        .unwrap_or(false);

    let explicit_positionals = command_attribute_bag.take("explicit_positionals")
        .map(expect_lit!(Lit::Bool))
        .transpose()?
        .map(|lit| lit.value)
        .unwrap_or(false);

    let paths_lits = command_cli_attributes.take_paths()?;

    command_attribute_bag.expect_empty()?;

    if !is_default && paths_lits.is_empty() {
        return Err(syn::Error::new_spanned(input.ident, "The command must have a path"));
    }

    if is_default {
        builder.push(quote! {
            builder.make_default();
        });
    }

    for path_lits in paths_lits {
        builder.push(quote! {
            builder.add_path(vec![#(#path_lits.to_string()),*]);
        });
    }

    for field in &mut struct_input.fields {
        let field_ident = &field.ident;

        let mut internal_field_type = &field.ty;
        let mut is_option_type = false;

        if let syn::Type::Path(type_path) = &internal_field_type {
            if &type_path.path.segments[0].ident == "Option" {
                let inner_type = &type_path.path.segments[0].arguments;
                if let syn::PathArguments::AngleBracketed(args) = inner_type {
                    if let syn::GenericArgument::Type(ty) = &args.args[0] {
                        internal_field_type = ty;
                        is_option_type = true;
                    }
                }
            }
        }

        let mut cli_attributes
            = CliAttributes::extract(&mut field.attrs)?;

        if !explicit_positionals && !cli_attributes.attributes.contains_key("option") && !cli_attributes.attributes.contains_key("positional") {
            cli_attributes.attributes.insert("positional".to_string(), vec![Attribute {
                pound_token: Default::default(),
                style: syn::AttrStyle::Outer,
                bracket_token: Default::default(),
                meta: Meta::Path(Path::from(Ident::new("positional", Span::call_site()))),
            }]);
        }

        if let Some(mut option_bag) = cli_attributes.take_unique::<OptionBag>("option")? {
            let mut is_bool = false;
            let mut arity = 1;

            if let syn::Type::Path(type_path) = &internal_field_type {
                if &type_path.path.segments[0].ident == "bool" {
                    is_bool = true;
                    arity = 0;
                }
            }
    
            if let syn::Type::Tuple(tuple) = internal_field_type {
                arity = tuple.elems.len();
            }

            let description = option_bag.attributes.take("help")
                .map(expect_lit!(Lit::Str))
                .transpose()?
                .map(|lit| lit.value())
                .unwrap_or_default();

            let is_required = option_bag.attributes.take("required")
                .map(expect_lit!(Lit::Bool))
                .transpose()?
                .map(|lit| lit.value)
                .unwrap_or(false);

            let preferred_name_lit = option_bag.path.iter()
                .max_by_key(|s| s.len())
                .map(to_lit_str)
                .unwrap();

            let name_set_lit = option_bag.path
                .iter()
                .map(to_lit_str)
                .collect::<Vec<_>>();

            let value_type = if arity > 0 {
                quote! {Array}
            } else if is_bool {
                quote! {Bool}
            } else {
                quote! {String}
            };

            let value_creator = match is_option_type {
                true => quote! {Some(value)},
                false => quote! {value},
            };

            if let Some(initial) = option_bag.attributes.take("initial") {
                default_hydrater.push(quote! {
                    self.#field_ident = #initial;
                });
            }

            option_hydrater.push(quote! {
                if option.0.as_str() == #preferred_name_lit {
                    if let clipanion::core::OptionValue::#value_type(value) = option.1 {
                        self.#field_ident = #value_creator;
                        continue;
                    }
                }
            });

            builder.push(quote! {
                builder.add_option(clipanion::core::OptionDefinition {
                    name_set: vec![#(#name_set_lit.to_string()),*],
                    description: #description.to_string(),
                    required: #is_required,
                    arity: #arity,
                    ..Default::default()
                })?;
            });

            option_bag.attributes.expect_empty()?;
        } else if let Some(positional_bag) = cli_attributes.take_unique::<AttributeBag>("positional")? {
            let field_name_upper = field.ident.as_ref().unwrap()
                .to_string()
                .to_uppercase();

            let mut is_rest = false;

            if let syn::Type::Path(type_path) = &internal_field_type {
                if &type_path.path.segments[0].ident == "Vec" {
                    is_rest = true;
                }
            }

            if is_rest {
                positional_hydrater.push(quote! {
                    if let clipanion::core::Positional::Rest(value) = positional {
                        self.#field_ident.push(value);
                        continue;
                    }
                });

                let add_cmd = match is_proxy {
                    true => quote! {add_proxy},
                    false => quote! {add_rest},
                };

                builder.push(quote! {
                    builder.#add_cmd(#field_name_upper)?;
                });
            } else {
                let value_creator = match is_option_type {
                    true => quote! {Some(value)},
                    false => quote! {value},
                };

                positional_hydrater.push(quote! {
                    if let clipanion::core::Positional::Required(value) = positional {
                        self.#field_ident = #value_creator;
                        continue;
                    }

                    if let clipanion::core::Positional::Optional(value) = positional {
                        self.#field_ident = #value_creator;
                        continue;
                    }
                });

                builder.push(quote! {
                    builder.add_positional(!#is_option_type, #field_name_upper)?;
                });
            }

            positional_bag.expect_empty()?;
        }
    }

    let struct_name = &input.ident;

    let expanded = quote! {
        #[derive(Default)]
        #input

        impl clipanion::details::CommandController for #struct_name {
            fn command_usage(opts: clipanion::core::CommandUsageOptions) -> Result<clipanion::core::CommandUsageResult, clipanion::core::BuildError> {
                let mut cli_builder = clipanion::core::CliBuilder::new();
                let mut builder = cli_builder.add_command();

                #(#builder)*

                Ok(builder.usage(opts))
            }

            fn attach_command_to_cli(builder: &mut clipanion::core::CommandBuilder) -> Result<(), clipanion::core::BuildError> {
                #(#builder)*
                Ok(())
            }

            fn hydrate_command_from_state(&mut self, state: clipanion::core::RunState) {
                #(#default_hydrater)*

                for option in state.options {
                    #(#option_hydrater)*
                }

                for positional in state.positionals {
                    #(#positional_hydrater)*
                }
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

#[proc_macro_attribute]
pub fn command(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match command_impl(args, input) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}
