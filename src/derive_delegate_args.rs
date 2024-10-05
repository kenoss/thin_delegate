use crate::punctuated_parser::PunctuatedParser;
use proc_macro2::{Span, TokenStream};
use quote::TokenStreamExt;
use syn::parse::Parse;

mod kw {
    syn::custom_keyword!(external_trait_def);
    syn::custom_keyword!(scheme);
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DeriveDelegateArgs {
    pub external_trait_def: Option<syn::Path>,
    pub scheme: Option<syn::ExprClosure>,
}

impl DeriveDelegateArgs {
    pub fn validate(&self) -> syn::Result<()> {
        let Some(scheme) = self.scheme.as_ref() else {
            return Ok(());
        };

        if !scheme.attrs.is_empty() {
            // This shouldn't happen because parsing of `syn::ExprClosure` requires that the stream
            // starts with `|...|`.
            panic!();
        }

        if scheme.lifetimes.is_some() {
            return Err(syn::Error::new_spanned(
                &scheme.lifetimes,
                "scheme can't have lifetime",
            ));
        }

        if scheme.constness.is_some() {
            return Err(syn::Error::new_spanned(
                scheme.constness,
                "scheme can't have `const`",
            ));
        }

        if scheme.movability.is_some() {
            return Err(syn::Error::new_spanned(
                scheme.movability,
                "scheme can't have `static`",
            ));
        }

        if scheme.asyncness.is_some() {
            return Err(syn::Error::new_spanned(
                scheme.asyncness,
                "scheme can't have `async`",
            ));
        }

        if scheme.capture.is_some() {
            return Err(syn::Error::new_spanned(
                scheme.capture,
                "scheme can't have `move`",
            ));
        }

        if scheme.inputs.len() != 1 {
            return Err(syn::Error::new_spanned(
                &scheme.inputs,
                "scheme must have an arg without type",
            ));
        }

        let syn::Pat::Ident(arg) = &scheme.inputs[0] else {
            return Err(syn::Error::new_spanned(
                &scheme.inputs[0],
                "scheme must have an arg without type",
            ));
        };

        if !arg.attrs.is_empty() {
            return Err(syn::Error::new_spanned(
                arg,
                "arg of scheme can't have attributes",
            ));
        }

        if arg.by_ref.is_some() {
            return Err(syn::Error::new_spanned(
                arg.by_ref,
                "arg of scheme can't have `ref`",
            ));
        }

        if arg.mutability.is_some() {
            return Err(syn::Error::new_spanned(
                arg.mutability,
                "arg of scheme can't have `mut`",
            ));
        }

        if arg.subpat.is_some() {
            let subpat = arg.subpat.as_ref().unwrap();
            let mut spans = TokenStream::new();
            spans.append_all(Some(&subpat.0));
            spans.append_all(Some(&subpat.1));
            return Err(syn::Error::new_spanned(
                spans,
                "arg of scheme can't have `@ SUBPATTERN`",
            ));
        }

        Ok(())
    }

    pub fn scheme_arg_and_body(&self) -> Option<(&syn::Ident, &syn::Expr)> {
        let Some(scheme) = &self.scheme else {
            return None;
        };

        let syn::Pat::Ident(arg) = &scheme.inputs[0] else {
            panic!();
        };

        Some((&arg.ident, &scheme.body))
    }
}

impl Parse for DeriveDelegateArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = DeriveDelegateArgs {
            external_trait_def: None,
            scheme: None,
        };

        let args = PunctuatedParser::<ParsableArg, syn::Token![,]>::parse(input)?;
        for arg in args.into_inner() {
            match arg {
                ParsableArg::ExternalTraitDef { path, .. } => {
                    this.external_trait_def = Some(path);
                }
                ParsableArg::Scheme { closure, .. } => {
                    this.scheme = Some(closure);
                }
            }
        }

        Ok(this)
    }
}

#[derive(Debug)]
enum ParsableArg {
    ExternalTraitDef {
        #[allow(unused)]
        external_trait_def_kw: kw::external_trait_def,
        #[allow(unused)]
        eq_token: syn::Token![=],
        path: syn::Path,
    },
    Scheme {
        #[allow(unused)]
        scheme_kw: kw::scheme,
        #[allow(unused)]
        eq_token: syn::Token![=],
        closure: syn::ExprClosure,
    },
}

impl Parse for ParsableArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::external_trait_def) {
            Ok(ParsableArg::ExternalTraitDef {
                external_trait_def_kw: input.parse()?,
                eq_token: input.parse()?,
                path: input.parse()?,
            })
        } else if lookahead.peek(kw::scheme) {
            Ok(ParsableArg::Scheme {
                scheme_kw: input.parse()?,
                eq_token: input.parse()?,
                closure: input.parse()?,
            })
        } else {
            Err(syn::Error::new(Span::call_site(), "error"))
        }
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
            scheme: None,
        };
        assert_eq!(syn::parse2::<DeriveDelegateArgs>(input).unwrap(), expected);

        let input = quote! { external_trait_def = __external_trait_def };
        let expected = DeriveDelegateArgs {
            external_trait_def: Some(parse_quote! { __external_trait_def }),
            scheme: None,
        };
        assert_eq!(syn::parse2::<DeriveDelegateArgs>(input).unwrap(), expected);

        let input = quote! { scheme = |f| f(&self.0.key()) };
        let expected = DeriveDelegateArgs {
            external_trait_def: None,
            scheme: Some(parse_quote! { |f| f(&self.0.key()) }),
        };
        assert_eq!(syn::parse2::<DeriveDelegateArgs>(input).unwrap(), expected);

        let input =
            quote! { external_trait_def = __external_trait_def, scheme = |f| f(&self.0.key()) };
        let expected = DeriveDelegateArgs {
            external_trait_def: Some(parse_quote! { __external_trait_def }),
            scheme: Some(parse_quote! { |f| f(&self.0.key()) }),
        };
        assert_eq!(syn::parse2::<DeriveDelegateArgs>(input).unwrap(), expected);

        assert!(syn::parse2::<DeriveDelegateArgs>(quote! { hoge = hoge }).is_err());
        assert!(syn::parse2::<DeriveDelegateArgs>(quote! { external_trait_def }).is_err());
        assert!(syn::parse2::<DeriveDelegateArgs>(
            quote! { external_trait_def = __external_trait_def,, }
        )
        .is_err());
    }

    #[test]
    fn validate() {
        macro_rules! assert_validate_error {
            ($input:expr, $expected_msg:expr) => {
                assert_eq!(
                    syn::parse2::<DeriveDelegateArgs>($input)
                        .unwrap()
                        .validate()
                        .map_err(|e| format!("{e}")),
                    $expected_msg.map_err(|e| e.to_string())
                );
            };
        }

        assert_validate_error!(
            quote! { scheme = for<'a> |f| f(&self.key()) },
            Err("scheme can't have lifetime")
        );

        assert_validate_error!(
            quote! { scheme = const |f| f(&self.key()) },
            Err("scheme can't have `const`")
        );

        assert_validate_error!(
            quote! { scheme = static |f| f(&self.key()) },
            Err("scheme can't have `static`")
        );

        assert_validate_error!(
            quote! { scheme = async |f| f(&self.key()) },
            Err("scheme can't have `async`")
        );

        assert_validate_error!(
            quote! { scheme = move |f| f(&self.key()) },
            Err("scheme can't have `move`")
        );

        assert_validate_error!(
            quote! { scheme = || f(&self.key()) },
            Err("scheme must have an arg without type")
        );

        assert_validate_error!(
            quote! { scheme = |f, g| f(&self.key()) },
            Err("scheme must have an arg without type")
        );

        assert_validate_error!(
            quote! { scheme = |f: usize| f(&self.key()) },
            Err("scheme must have an arg without type")
        );

        assert_validate_error!(
            quote! { scheme = |#[cfg(all())] f| f(&self.key()) },
            Err("arg of scheme can't have attributes")
        );

        assert_validate_error!(
            quote! { scheme = |ref f| f(&self.key()) },
            Err("arg of scheme can't have `ref`")
        );

        assert_validate_error!(
            quote! { scheme = |mut f| f(&self.key()) },
            Err("arg of scheme can't have `mut`")
        );

        assert_validate_error!(
            quote! { scheme = |f @ Fuga| f(&self.key()) },
            Err("arg of scheme can't have `@ SUBPATTERN`")
        );
    }
}
