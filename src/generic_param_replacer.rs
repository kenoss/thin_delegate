use indoc::indoc;
use itertools::izip;
use quote::ToTokens;
use std::collections::HashMap;
use syn::visit_mut::VisitMut;

pub(crate) struct GenericParamReplacer {
    lifetimes: HashMap<syn::Lifetime, syn::Lifetime>,
    types: HashMap<syn::TypePath, syn::Type>,
    exprs: HashMap<syn::Path, syn::Expr>,
}

impl GenericParamReplacer {
    pub fn new(orig: &syn::Generics, subst: &syn::PathArguments) -> syn::Result<Self> {
        #![allow(clippy::single_match)]

        let mut this = Self {
            lifetimes: HashMap::new(),
            types: HashMap::new(),
            exprs: HashMap::new(),
        };

        let subst = match (orig.params.len(), subst) {
            (0, syn::PathArguments::None) => {
                return Ok(this);
            }
            (_, syn::PathArguments::None) => {
                return Err(syn::Error::new_spanned(
                    subst,
                    format!(
                        indoc! {r#"
                            number of generic parameters must coinside:
                                in definition of trait         = {orig}
                                in definition of derive target = {subst}
                        "#},
                        orig = orig.to_token_stream(),
                        subst = subst.to_token_stream(),
                    ),
                ));
            }
            (_, syn::PathArguments::AngleBracketed(subst)) => subst,
            _ => {
                return Err(syn::Error::new_spanned(
                    subst,
                    "expected generic arguments `<...>`",
                ));
            }
        };

        if orig.params.len() != subst.args.len() {
            return Err(syn::Error::new_spanned(
                subst,
                format!(
                    indoc! {r#"
                        number of generic parameters must coinside:
                            in definition of trait         = {orig}
                            in definition of derive target = {subst}
                    "#},
                    orig = orig.to_token_stream(),
                    subst = subst.to_token_stream(),
                ),
            ));
        }

        for (o, s) in izip!(orig.params.iter(), subst.args.iter()) {
            match o {
                syn::GenericParam::Lifetime(o) => {
                    let syn::GenericArgument::Lifetime(s) = s else {
                        return Err(syn::Error::new_spanned(
                            s,
                            format!(
                                indoc! {r#"
                                    parameter can't be substituted to argument:
                                        in definition of trait         = {orig}
                                        in definition of derive target = {subst}
                                "#},
                                orig = orig.to_token_stream(),
                                subst = subst.to_token_stream(),
                            ),
                        ));
                    };
                    this.lifetimes.insert(o.lifetime.clone(), s.clone());
                }
                syn::GenericParam::Type(o) => {
                    let o_path = syn::Path::from(syn::PathSegment::from(o.ident.clone()));
                    match s {
                        // s/T/S/ in `[T; N]` -> `[S; N]`
                        syn::GenericArgument::Type(s) => {
                            let type_path = syn::TypePath {
                                qself: None,
                                path: o_path,
                            };
                            this.types.insert(type_path, s.clone());
                        }
                        _ => {}
                    }
                }
                syn::GenericParam::Const(o) => {
                    // In `trait Trait<const N: usize>`,
                    let o_path = syn::Path::from(syn::PathSegment::from(o.ident.clone()));
                    match s {
                        // s/N/4/ in `[T; N]` -> `[T; 4]`
                        syn::GenericArgument::Const(s) => {
                            this.exprs.insert(o_path.clone(), s.clone());
                        }
                        // s/N/M/ in `[T; N]` -> `[T; M]`
                        syn::GenericArgument::Type(syn::Type::Path(s)) => {
                            let expr = syn::Expr::Path(syn::ExprPath {
                                attrs: vec![],
                                qself: None,
                                path: s.path.clone(),
                            });
                            this.exprs.insert(o_path, expr);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(this)
    }

    pub fn replace_signature(&self, mut sig: syn::Signature) -> syn::Signature {
        let mut visitor = Visitor(self);
        visitor.visit_signature_mut(&mut sig);
        sig
    }
}

struct Visitor<'a>(&'a GenericParamReplacer);

// Use `visit_*_mut()` as we may need to change enum variant when it matches.
impl VisitMut for Visitor<'_> {
    fn visit_expr_mut(&mut self, node: &mut syn::Expr) {
        syn::visit_mut::visit_expr_mut(self, node);

        #[allow(clippy::single_match)]
        match node {
            syn::Expr::Path(expr_path) => {
                if let Some(subst) = self.0.exprs.get(&expr_path.path) {
                    *node = subst.clone();
                }
            }
            _ => {}
        }
    }

    fn visit_lifetime_mut(&mut self, node: &mut syn::Lifetime) {
        if let Some(subst) = self.0.lifetimes.get(node) {
            *node = subst.clone();
        }
    }

    fn visit_type_mut(&mut self, node: &mut syn::Type) {
        syn::visit_mut::visit_type_mut(self, node);

        #[allow(clippy::single_match)]
        match node {
            syn::Type::Path(x) => {
                if let Some(subst) = self.0.types.get(x) {
                    *node = subst.clone();
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    macro_rules! test_replace_signature {
        (
            $test_name:ident,
            $orig:expr,
            $func:expr,
            $subst:expr,
            $func_replaced_expected:expr,
        ) => {
            #[test]
            fn $test_name() -> Result<(), syn::Error> {
                let orig = $orig;
                let orig = syn::parse2::<syn::ItemTrait>(quote! { trait #orig {} }).unwrap();
                let orig = &orig.generics;
                let subst = syn::parse2::<syn::PathSegment>($subst).unwrap();
                let subst = &subst.arguments;
                let func = syn::parse2::<syn::TraitItemFn>($func).unwrap();
                let func_replaced_expected =
                    syn::parse2::<syn::TraitItemFn>($func_replaced_expected).unwrap();

                let generic_param_replacer = GenericParamReplacer::new(&orig, &subst)?;
                let got = generic_param_replacer.replace_signature(func.sig.clone());
                assert_eq!(
                    got,
                    func_replaced_expected.sig,
                    "\n    got      = {},\n    expected = {}",
                    got.to_token_stream(),
                    func_replaced_expected.sig.to_token_stream(),
                );

                Ok(())
            }
        };
    }

    test_replace_signature! {
        type_path,
        quote! { AsRef<T> },
        quote! { fn as_ref(&self) -> &T; },
        quote! { AsRef<str> },
        quote! { fn as_ref(&self) -> &str; },
    }

    test_replace_signature! {
        type_path_type,
        quote! { Hello<T> },
        quote! { fn hello(&self) -> Vec<T>; },
        quote! { Hello<&str> },
        quote! { fn hello(&self) -> Vec<&str>; },
    }

    test_replace_signature! {
        type_array_type,
        quote! { Hello<T> },
        quote! { fn hello(&self) -> [T; 4]; },
        quote! { Hello<u8> },
        quote! { fn hello(&self) -> [u8; 4]; },
    }

    test_replace_signature! {
        type_slice_type,
        quote! { Hello<T> },
        quote! { fn hello(&self) -> &[T]; },
        quote! { Hello<u8> },
        quote! { fn hello(&self) -> &[u8]; },
    }

    test_replace_signature! {
        lifetime,
        quote! { Hello<'a, T> },
        quote! { fn hello(&self) -> &'a T; },
        quote! { Hello<'p, str> },
        quote! { fn hello(&self) -> &'p str; },
    }

    test_replace_signature! {
        const_expr,
        // trait Hello<T, const N: usize> { ... }
        quote! { Hello<T, const N: usize> },
        quote! { fn hello(&self) -> [T; N]; },
        // struct Hoge { ... }
        // impl Hello<u8, 4> for Hoge { ... }
        quote! { Hello<u8, 4> },
        quote! { fn hello(&self) -> [u8; 4]; },
    }

    test_replace_signature! {
        const_path,
        // trait Hello<T, const N: usize> { ... }
        quote! { Hello<T, const N: usize> },
        quote! { fn hello(&self) -> [T; N]; },
        // struct Hoge<const M: usize> { ... }
        // impl<const M: usize> Hello<u8, 4> for Hoge<M> { ... }
        quote! { Hello<u8, M> },
        quote! { fn hello(&self) -> [u8; M]; },
    }
}
