extern crate proc_macro;

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, punctuated::Punctuated, Attribute, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, LitBool, LitStr, Meta, Path, Token};

macro_rules! expect_lit {
    ($expression:path) => {
        |val| match val {
            Expr::Lit(ExprLit {lit: $expression(value), ..}) => Ok(value),
            _ => Err(syn::Error::new_spanned(val, "Invalid literal type")),
        }
    };
}

fn to_lit_str<T: AsRef<str>>(str: T) -> LitStr {
    LitStr::new(str.as_ref(), proc_macro2::Span::call_site())
}

#[derive(Clone, Default)]
struct AttributeBag {
    attributes: HashMap<String, Expr>,
}

impl AttributeBag {
    pub fn expect_empty(&self) -> syn::Result<()> {
        if !self.attributes.is_empty() {
            return Err(syn::Error::new_spanned(self.attributes.iter().next().unwrap().1, "Unsupported extra attributes"));
        }

        Ok(())
    }

    pub fn take(&mut self, key: &str) -> Option<Expr> {
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

                let value: Expr = input.parse()?;
                attributes.insert(name, value);
            } else {
                attributes.insert(name, Expr::Lit(ExprLit {
                    attrs: vec![],
                    lit: Lit::Bool(LitBool {
                        value: true,
                        span: proc_macro2::Span::call_site(),
                    }),
                }));
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

        let mut path = path.value()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();

        path.sort_by(|a, b| {
            a.len().cmp(&b.len())
        });

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
            if path.segments.is_empty() || path.segments[0].ident != "cli" {
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

    let paths_lits
        = command_cli_attributes.take_paths()?;

    command_attribute_bag.expect_empty()?;

    let mut partial_struct_members
        = vec![];

    let mut initialization_members
        = vec![];

    if !is_default && paths_lits.is_empty() {
        return Err(syn::Error::new_spanned(input.ident, "The command must have a path"));
    }

    if is_default {
        builder.push(quote! {
            command_spec.paths.push(vec![]);
        });
    }

    for path_lits in paths_lits {
        builder.push(quote! {
            command_spec.paths.push(vec![#(#path_lits.to_string()),*]);
        });
    }

    for field in &mut struct_input.fields {
        let field_ident = &field.ident;
        let field_type = &field.ty;

        let mut internal_field_type = &field.ty;
        let mut is_option_type = false;
        let mut is_vec_type = false;

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

            if &type_path.path.segments[0].ident == "Vec" {
                let inner_type = &type_path.path.segments[0].arguments;
                if let syn::PathArguments::AngleBracketed(args) = inner_type {
                    if let syn::GenericArgument::Type(ty) = &args.args[0] {
                        internal_field_type = ty;
                        is_vec_type = true;
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

            let mut min_len = 1usize;
            let mut extra_len = Some(0usize);

            if let syn::Type::Path(type_path) = &field_type {
                if &type_path.path.segments[0].ident == "bool" {
                    is_bool = true;

                    min_len = 0;
                    extra_len = Some(0);
                }
            }

            if let syn::Type::Tuple(tuple) = internal_field_type {
                let tuple_len = tuple.elems.len();

                min_len = tuple_len;
                extra_len = Some(0);
            }

            if is_vec_type {
                min_len = 0;
                extra_len = None;
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

            let preferred_name_lit = option_bag.path
                .first()
                .map(to_lit_str)
                .unwrap();

            let aliases_lit = option_bag.path
                .iter()
                .skip(1)
                .map(to_lit_str)
                .collect::<Vec<_>>();

            let min_len_lit = quote! {#min_len};
            let extra_len_lit = match extra_len {
                Some(extra_len) => quote! {Some(#extra_len)},
                None => quote! {None},
            };

            let value_converter = if min_len > 1 {
                quote! {values.iter().map(|s| s.parse().map_err(clipanion::details::handle_parse_error)).collect::<Result<Vec<_>, _>>()?}
            } else if is_bool {
                quote! {true}
            } else {
                quote! {values.first().map(|s| s.parse().map_err(clipanion::details::handle_parse_error)).transpose()?}
            };

            let default_value
                = option_bag.attributes.take("default");

            if is_vec_type {
                partial_struct_members.push(quote! {
                    #field_ident: Vec<#internal_field_type>,
                });

                option_hydrater.push(quote! {
                    partial.#field_ident = #value_converter;
                });

                initialization_members.push(quote! {
                    #field_ident: Vec::new(),
                });
            } else {
                partial_struct_members.push(quote! {
                    #field_ident: Option<#field_type>,
                });

                if is_option_type {
                    option_hydrater.push(quote! {
                        partial.#field_ident = Some(#value_converter);
                    });
                } else {
                    option_hydrater.push(quote! {
                        partial.#field_ident = Some(#value_converter.unwrap());
                    });
                }

                let accessor = match default_value {
                    Some(expr) => quote! { partial.#field_ident.or_else(|| Some(#expr)) },
                    None => quote! { partial.#field_ident },
                };

                if is_option_type {
                    initialization_members.push(quote! {
                        #field_ident: #accessor.unwrap_or_default(),
                    });
                } else {
                    initialization_members.push(quote! {
                        #field_ident: #accessor.unwrap(),
                    });
                }
            }

            builder.push(quote! {
                command_spec.components.push(clipanion::core::Component::Option(clipanion::core::OptionSpec {
                    primary_name: #preferred_name_lit.to_string(),
                    aliases: vec![#(#aliases_lit.to_string()),*],
                    description: #description.to_string(),
                    is_hidden: false,
                    is_required: #is_required,
                    allow_binding: false,
                    min_len: #min_len_lit,
                    extra_len: #extra_len_lit,
                }));
            });

            option_bag.attributes.expect_empty()?;
        } else if let Some(positional_bag) = cli_attributes.take_unique::<AttributeBag>("positional")? {
            let field_name_upper = field.ident.as_ref().unwrap()
                .to_string()
                .to_uppercase();

            if is_vec_type {
                partial_struct_members.push(quote! {
                    #field_ident: Vec<#internal_field_type>,
                });
    
                positional_hydrater.push(quote! {
                    let value = args.iter()
                        .map(|arg| arg.parse().map_err(clipanion::details::handle_parse_error))
                        .collect::<Result<Vec<_>, _>>()?;

                    partial.#field_ident = value;
                });

                initialization_members.push(quote! {
                    #field_ident: partial.#field_ident,
                });

                builder.push(quote! {
                    command_spec.components.push(clipanion::core::Component::Positional(clipanion::core::PositionalSpec::Dynamic {
                        name: #field_name_upper.to_string(),
                        description: "".to_string(),
                        min_len: 0,
                        extra_len: None,
                    }));
                });
            } else {
                partial_struct_members.push(quote! {
                    #field_ident: Option<#field_type>,
                });

                if is_option_type {
                    positional_hydrater.push(quote! {
                        let positional = args.first().unwrap();

                        let value = positional.parse()
                            .map_err(clipanion::details::handle_parse_error)?;

                        partial.#field_ident = Some(Some(value));
                    });
                } else {
                    positional_hydrater.push(quote! {
                        let positional = args.first().unwrap();

                        let value = positional.parse()
                            .map_err(clipanion::details::handle_parse_error)?;

                        partial.#field_ident = Some(value);
                    });
                }

                initialization_members.push(quote! {
                    #field_ident: partial.#field_ident.unwrap(),
                });

                let (min_len, extra_len) = match is_option_type {
                    true => (quote!{0}, quote!{Some(1)}),
                    false => (quote!{1}, quote!{Some(0)}),
                };

                builder.push(quote! {
                    command_spec.components.push(clipanion::core::Component::Positional(clipanion::core::PositionalSpec::Dynamic {
                        name: #field_name_upper.to_string(),
                        description: "".to_string(),
                        min_len: #min_len,
                        extra_len: #extra_len,
                    }));
                });
            }

            positional_bag.expect_empty()?;
        }
    }

    if let Fields::Named(fields) = &mut struct_input.fields {
        fields.named.push(syn::parse_quote! {cli_environment: clipanion::advanced::Environment});
        fields.named.push(syn::parse_quote! {cli_path: Vec<String>});
    }

    let struct_name
        = &input.ident;

    let expanded = quote! {
        #input

        impl clipanion::details::CommandController for #struct_name {
            fn command_usage(opts: clipanion::core::CommandUsageOptions) -> Result<clipanion::core::CommandUsageResult, clipanion::core::BuildError> {
                Ok(#struct_name::command_spec()?.usage())
            }

            fn command_spec() -> Result<clipanion::core::CommandSpec, clipanion::core::BuildError> {
                let mut command_spec
                    = clipanion::core::CommandSpec::default();

                #(#builder)*

                Ok(command_spec)
            }

            fn hydrate_command_from_state(environment: &clipanion::advanced::Environment, state: &clipanion::core::State) -> Result<Self, clipanion::core::CommandError> {
                #[derive(Default, Debug)]
                struct Partial {
                    #(#partial_struct_members)*
                }

                let mut partial
                    = Partial::default();

                let FNS: &[fn(&mut Partial, &[&str]) -> Result<(), clipanion::core::CustomError>] = &[
                    #(|partial, args| {
                        #positional_hydrater
                        Ok(())
                    }),*
                ];
    
                for (index, args) in &state.option_values {
                    FNS[*index](&mut partial, args)?;
                }

                for (index, args) in &state.positional_values {
                    FNS[*index](&mut partial, args)?;
                }

                Ok(Self {
                    cli_environment: environment.clone(),
                    cli_path: state.path.iter().map(|s| s.to_string()).collect(),

                    #(#initialization_members)*
                })
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
