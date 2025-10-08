use quote::format_ident;

macro_rules! expect_lit {
    ($expression:path) => {
        |val| match val {
            Expr::Lit(ExprLit {lit: $expression(value), ..}) => Ok(value),
            _ => Err(syn::Error::new_spanned(val, "Invalid literal type")),
        }
    };
}

pub(crate) use expect_lit;

pub fn get_partial_enum_ident(enum_ident: &syn::Ident) -> syn::Ident {
    format_ident!("Partial{}", enum_ident)
}
