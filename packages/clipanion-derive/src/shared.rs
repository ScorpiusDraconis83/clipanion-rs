use quote::format_ident;

pub fn get_cli_enum_names(cli_ident: &syn::Ident) -> (syn::Ident, syn::Ident) {
    let partial_enum_ident
        = format_ident!("{}PartialEnum", cli_ident);

    let enum_ident
        = format_ident!("{}Enum", cli_ident);

    (partial_enum_ident, enum_ident)
}

pub fn get_command_variant_ident(index: usize, _: &syn::Path) -> syn::Ident {
    format_ident!("_Variant{}", index + 1)
}
