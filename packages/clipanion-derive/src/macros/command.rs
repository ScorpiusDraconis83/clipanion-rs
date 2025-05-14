use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, LitStr, Meta, Path};

use crate::utils::{to_lit_str, AttributeBag, CliAttributes, OptionBag};

macro_rules! expect_lit {
    ($expression:path) => {
        |val| match val {
            Expr::Lit(ExprLit {lit: $expression(value), ..}) => Ok(value),
            _ => Err(syn::Error::new_spanned(val, "Invalid literal type")),
        }
    };
}

pub fn command_macro(args: TokenStream, mut input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let syn::Data::Struct(struct_input) = &mut input.data else {
        panic!("Only structs are supported");
    };

    let partial_struct_ident
        = Ident::new(&format!("Partial{}", input.ident), Span::call_site());

    let mut builder = vec![];
    let mut hydraters = vec![];
    
    let mut command_cli_attributes
        = CliAttributes::extract(&mut input.attrs)?;

    let mut command_attribute_bag
        = syn::parse::<AttributeBag>(args)?;

    let command_category = command_cli_attributes
        .take_unique::<LitStr>("category")?
        .map(|lit| quote!{command_spec.category = Some(#lit.to_string());});

    let command_description = command_cli_attributes
        .take_unique::<LitStr>("description")?
        .map(|lit| quote!{command_spec.description = Some(#lit.to_string());});

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
    let mut partial_struct_default_initializers
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
            let mut is_tuple = false;

            let mut item_count = 1usize;
            let mut min_len = 1usize;
            let mut extra_len = Some(0usize);

            if let syn::Type::Path(type_path) = &internal_field_type {
                if &type_path.path.segments[0].ident == "bool" {
                    is_bool = true;

                    min_len = 0;
                    extra_len = Some(0);
                }
            }

            if let syn::Type::Tuple(tuple) = internal_field_type {
                let tuple_len = tuple.elems.len();

                is_tuple = true;

                item_count = tuple_len;
                min_len = tuple_len;
                extra_len = Some(0);
            }

            if is_vec_type {
                min_len = 0;
                extra_len = None;

                if let Some(min_len_expr) = option_bag.attributes.take("min_len") {
                    let min_len_lit
                        = expect_lit!(Lit::Int)(min_len_expr)?;

                    min_len = min_len_lit
                        .base10_parse::<usize>().unwrap_or(0);
                }
            }

            let description = option_bag.attributes.take("help")
                .map(expect_lit!(Lit::Str))
                .transpose()?
                .map(|lit| lit.value())
                .unwrap_or_default();

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

            let value_converter = if is_vec_type {
                if is_tuple {
                    let mut tuple_fields = vec![];

                    for field in 0..item_count {
                        tuple_fields.push(quote! {
                            args.get(#field).map(|s| -> Result<_, clipanion::core::CommandError> {
                                s.parse().map_err(clipanion::details::handle_parse_error)
                            }).transpose()?.unwrap()
                        });
                    }
    
                    quote! {
                        args.chunks(#item_count).map(|chunk| -> Result<_, clipanion::core::CommandError> {
                            Ok((#(#tuple_fields),*))
                        }).collect::<Result<Vec<_>, _>>()?
                    }
                } else {
                    quote! {args.iter().map(|s| -> Result<_, clipanion::core::CommandError> {
                        s.parse().map_err(clipanion::details::handle_parse_error)
                    }).collect::<Result<Vec<_>, _>>()?}
                }
            } else if is_bool {
                quote! {Some(true)}
            } else if is_tuple {
                let mut tuple_fields
                    = vec![];

                for field in 0..item_count {
                    tuple_fields.push(quote! {
                        args.get(#field).map(|s| -> Result<_, clipanion::core::CommandError> {
                            s.parse().map_err(clipanion::details::handle_parse_error)
                        }).transpose()?.unwrap()
                    });
                }

                quote! {Some((#(#tuple_fields),*))}
            } else {
                quote! {args.first().map(|s| -> Result<_, clipanion::core::CommandError> {
                    s.parse().map_err(clipanion::details::handle_parse_error)
                }).transpose()?}
            };

            let none_expr: Expr
                = syn::parse_str("None").unwrap();

            let default_value
                = option_bag.attributes.take("default")
                    .or_else(|| if is_option_type {Some(none_expr.clone())} else {None});

            let is_required
                = default_value.is_none();

            if is_vec_type {
                partial_struct_members.push(quote! {
                    #field_ident: Vec<#internal_field_type>,
                });

                partial_struct_default_initializers.push(quote! {
                    #field_ident: std::default::Default::default(),
                });

                hydraters.push(quote! {
                    partial.#field_ident.extend(#value_converter);
                });

                initialization_members.push(quote! {
                    #field_ident: partial.#field_ident,
                });
            } else {
                hydraters.push(quote! {
                    partial.#field_ident = #value_converter;
                });

                // If the field is an option type we already assume it's an
                // optional option; we don't need to wrap it inside Option<T>
                let partial_struct_member_type = match is_option_type {
                    true => quote! {#field_type},
                    false => quote! {Option<#field_type>},
                };

                partial_struct_members.push(quote! {
                    #field_ident: #partial_struct_member_type,
                });

                partial_struct_default_initializers.push(quote! {
                    #field_ident: std::default::Default::default(),
                });

                let option_partial = match is_option_type {
                    true => quote! {Some(partial.#field_ident)},
                    false => quote! {partial.#field_ident},
                };

                let partial_to_ty = match default_value {
                    Some(expr) => quote! {#option_partial.unwrap_or_else(|| #expr)},
                    None => quote! {#option_partial.unwrap()},
                };

                initialization_members.push(quote! {
                    #field_ident: #partial_to_ty,
                });
            }

            builder.push(quote! {
                if #is_required {
                    command_spec.required_options.push(command_spec.components.len());
                }

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
        } else if let Some(mut positional_bag) = cli_attributes.take_unique::<AttributeBag>("positional")? {
            let description = positional_bag.take("help")
                .map(expect_lit!(Lit::Str))
                .transpose()?
                .map(|lit| lit.value())
                .unwrap_or_default();

            let is_prefix = positional_bag.take("is_prefix")
                .map(expect_lit!(Lit::Bool))
                .transpose()?
                .map(|lit| lit.value())
                .unwrap_or_default();

            let field_name_upper = field.ident.as_ref().unwrap()
                .to_string()
                .to_uppercase();

            if is_vec_type {
                partial_struct_members.push(quote! {
                    #field_ident: Vec<#internal_field_type>,
                });

                partial_struct_default_initializers.push(quote! {
                    #field_ident: std::default::Default::default(),
                });

                hydraters.push(quote! {
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
                        description: #description.to_string(),
                        min_len: 0,
                        extra_len: None,
                        is_prefix: #is_prefix,
                        is_proxy: #is_proxy,
                    }));
                });
            } else {
                partial_struct_members.push(quote! {
                    #field_ident: Option<#field_type>,
                });

                partial_struct_default_initializers.push(quote! {
                    #field_ident: std::default::Default::default(),
                });

                if is_option_type {
                    hydraters.push(quote! {
                        let positional = args.first().unwrap();

                        let value = positional.parse()
                            .map_err(clipanion::details::handle_parse_error)?;

                        partial.#field_ident = Some(Some(value));
                    });

                    initialization_members.push(quote! {
                        #field_ident: partial.#field_ident.unwrap_or_default(),
                    });
                } else {
                    hydraters.push(quote! {
                        let positional = args.first().unwrap();

                        let value = positional.parse()
                            .map_err(clipanion::details::handle_parse_error)?;

                        partial.#field_ident = Some(value);
                    });

                    initialization_members.push(quote! {
                        #field_ident: partial.#field_ident.unwrap(),
                    });
                }

                let (min_len, extra_len) = match is_option_type {
                    true => (quote!{0}, quote!{Some(1)}),
                    false => (quote!{1}, quote!{Some(0)}),
                };

                builder.push(quote! {
                    command_spec.components.push(clipanion::core::Component::Positional(clipanion::core::PositionalSpec::Dynamic {
                        name: #field_name_upper.to_string(),
                        description: #description.to_string(),
                        min_len: #min_len,
                        extra_len: #extra_len,
                        is_prefix: #is_prefix,
                        is_proxy: false,
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

        #[derive(Debug)]
        pub struct #partial_struct_ident {
            cli_environment: clipanion::advanced::Environment,
            cli_path: Vec<String>,

            #(#partial_struct_members)*
        }

        impl #partial_struct_ident {
            pub fn new(environment: &clipanion::advanced::Environment, path: Vec<String>) -> Self {
                Self {
                    cli_environment: environment.clone(),
                    cli_path: path,

                    #(#partial_struct_default_initializers)*
                }
            }
        }

        impl TryFrom<#partial_struct_ident> for #struct_name {
            type Error = ::clipanion::core::CommandError;

            fn try_from(partial: #partial_struct_ident) -> Result<Self, ::clipanion::core::CommandError> {
                Ok(Self {
                    cli_environment: partial.cli_environment,
                    cli_path: partial.cli_path,

                    #(#initialization_members)*
                })
            }
        }

        impl clipanion::details::CommandController for #struct_name {
            type Partial = #partial_struct_ident;

            fn command_usage(opts: clipanion::core::CommandUsageOptions) -> Result<clipanion::core::CommandUsageResult, clipanion::core::BuildError> {
                Ok(#struct_name::command_spec()?.usage())
            }

            fn command_spec() -> Result<&'static clipanion::core::CommandSpec, clipanion::core::BuildError> {
                use std::ops::Deref;
                use std::sync::LazyLock;

                static COMMAND_SPEC: LazyLock<Result<clipanion::core::CommandSpec, clipanion::core::BuildError>> = LazyLock::new(|| {
                    let mut command_spec
                        = clipanion::core::CommandSpec::default();

                    #command_category
                    #command_description

                    #(#builder)*

                    Ok(command_spec)
                });

                COMMAND_SPEC.deref().as_ref().map_err(|e| e.clone())
            }

            fn hydrate_from_state(environment: &clipanion::advanced::Environment, state: &clipanion::core::State) -> Result<Self::Partial, clipanion::core::CommandError> {
                let mut partial
                    = Self::Partial::new(environment, state.path.iter().map(|s| s.to_string()).collect());

                let FNS: &[fn(&mut Self::Partial, &[&str]) -> Result<(), clipanion::core::CommandError>] = &[
                    #(|partial, args| {
                        #hydraters
                        Ok(())
                    }),*
                ];

                for (index, args) in &state.option_values {
                    FNS[*index](&mut partial, args)?;
                }

                for (index, args) in &state.positional_values {
                    FNS[*index](&mut partial, args)?;
                }

                Ok(partial)
            }
        }
    };

    Ok(TokenStream::from(expanded))
}
