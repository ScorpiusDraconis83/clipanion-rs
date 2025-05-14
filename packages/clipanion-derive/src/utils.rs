use std::collections::HashMap;

use syn::{parse::{Parse, ParseStream}, punctuated::Punctuated, Attribute, Expr, ExprLit, Ident, Lit, LitBool, LitStr, Meta, Token};

pub fn to_lit_str<T: AsRef<str>>(str: T) -> LitStr {
    LitStr::new(str.as_ref(), proc_macro2::Span::call_site())
}

#[derive(Clone, Default)]
pub struct AttributeBag {
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
pub struct OptionBag {
    pub path: Vec<String>,
    pub attributes: AttributeBag,
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
pub struct CliAttributes {
    pub attributes: HashMap<String, Vec<Attribute>>,
}

impl CliAttributes {
    fn parse_args<T: Parse>(attr: &Attribute) -> syn::Result<T> {
        match attr.meta {
            Meta::Path(_) => {
                let meta = Meta::List(syn::MetaList {
                    path: syn::Path::from(Ident::new("positional", proc_macro2::Span::call_site())),
                    delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                    tokens: proc_macro2::TokenStream::new(),
                });

                let attribute = Attribute {
                    pound_token: Default::default(),
                    style: syn::AttrStyle::Outer,
                    bracket_token: Default::default(),
                    meta: meta,
                };

                attribute.parse_args::<T>()
            },

            _ => {
                attr.parse_args::<T>()
            },
        }
    }

    pub fn extract(attrs: &mut Vec<Attribute>) -> syn::Result<Self> {
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

    pub fn take_unique<T: Parse>(&mut self, key: &str) -> syn::Result<Option<T>> {
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

    pub fn take_paths(&mut self) -> syn::Result<Vec<Vec<LitStr>>> {
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
