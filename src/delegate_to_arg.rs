use syn::parse::{Parse, ParseStream};

pub(crate) struct DelegateToArg {
    pub ident: syn::Ident,
    #[allow(unused)]
    pub fat_arrow_token: syn::token::FatArrow,
    pub expr: syn::Expr,
}

impl Parse for DelegateToArg {
    fn parse(input: ParseStream) -> syn::Result<DelegateToArg> {
        Ok(DelegateToArg {
            ident: input.parse()?,
            fat_arrow_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parsable() {
        syn::parse2::<DelegateToArg>(quote! { x => &x.0 }).unwrap();
        syn::parse2::<DelegateToArg>(quote! { x => x::x(&x.0) }).unwrap();
    }
}
