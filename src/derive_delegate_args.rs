use crate::punctuated_parser::PunctuatedParser;
use syn::parse::Parse;

mod kw {
    syn::custom_keyword!(external_trait_def);
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DeriveDelegateArgs {
    pub external_trait_def: Option<syn::Path>,
}

impl Parse for DeriveDelegateArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = DeriveDelegateArgs {
            external_trait_def: None,
        };

        let args = PunctuatedParser::<ParsableArg, syn::Token![,]>::parse(input)?;
        for arg in args.into_inner() {
            this.external_trait_def = Some(arg.path);
        }

        Ok(this)
    }
}

#[derive(Debug)]
struct ParsableArg {
    #[allow(unused)]
    external_trait_def_kw: kw::external_trait_def,
    #[allow(unused)]
    eq_token: syn::Token![=],
    path: syn::Path,
}

impl Parse for ParsableArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(ParsableArg {
            external_trait_def_kw: input.parse()?,
            eq_token: input.parse()?,
            path: input.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn parsable() {
        let input = quote! {};
        let expected = DeriveDelegateArgs {
            external_trait_def: None,
        };
        assert_eq!(syn::parse2::<DeriveDelegateArgs>(input).unwrap(), expected,);

        let input = quote! { external_trait_def = __external_trait_def };
        let expected = DeriveDelegateArgs {
            external_trait_def: Some(parse_quote! { __external_trait_def }),
        };
        assert_eq!(syn::parse2::<DeriveDelegateArgs>(input).unwrap(), expected,);

        assert!(syn::parse2::<DeriveDelegateArgs>(quote! { hoge = hoge }).is_err());
    }
}
