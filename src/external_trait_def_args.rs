use syn::parse::Parse;

mod kw {
    syn::custom_keyword!(with_uses);
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ExternalTraitDefArgs {
    pub with_uses: bool,
}

impl Parse for ExternalTraitDefArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = ExternalTraitDefArgs { with_uses: false };

        let args =
            syn::punctuated::Punctuated::<ParsableArg, syn::Token![,]>::parse_terminated(input)?;
        for arg in args {
            match arg {
                ParsableArg::WithUses { with_uses, .. } => {
                    this.with_uses = with_uses.value;
                }
            }
        }

        Ok(this)
    }
}

#[derive(Debug)]
enum ParsableArg {
    WithUses {
        #[allow(unused)]
        with_uses_kw: kw::with_uses,
        #[allow(unused)]
        eq_token: syn::Token![=],
        with_uses: syn::LitBool,
    },
}

impl Parse for ParsableArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::with_uses) {
            Ok(ParsableArg::WithUses {
                with_uses_kw: input.parse()?,
                eq_token: input.parse()?,
                with_uses: input.parse()?,
            })
        } else {
            Err(input.error("expected `with_uses`"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parsable() {
        let input = quote! {};
        let expected = ExternalTraitDefArgs { with_uses: false };
        assert_eq!(
            syn::parse2::<ExternalTraitDefArgs>(input).unwrap(),
            expected
        );

        let input = quote! { with_uses = true };
        let expected = ExternalTraitDefArgs { with_uses: true };
        assert_eq!(
            syn::parse2::<ExternalTraitDefArgs>(input).unwrap(),
            expected
        );

        assert!(syn::parse2::<ExternalTraitDefArgs>(quote! { hoge = hoge }).is_err());
    }
}
