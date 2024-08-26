use syn::visit_mut::VisitMut;

pub(crate) fn replace_ident_in_expr(
    orig: syn::Ident,
    subst: syn::Ident,
    mut target: syn::Expr,
) -> syn::Expr {
    let mut visitor = Visitor { orig, subst };
    visitor.visit_expr_mut(&mut target);
    target
}

struct Visitor {
    orig: syn::Ident,
    subst: syn::Ident,
}

// Use `visit_*_mut()` as we may need to change enum variant when it matches.
impl VisitMut for Visitor {
    fn visit_expr_mut(&mut self, node: &mut syn::Expr) {
        syn::visit_mut::visit_expr_mut(self, node);

        #[allow(clippy::single_match)]
        match node {
            syn::Expr::Path(expr_path) => {
                if expr_path.path.is_ident(&self.orig) {
                    let path = syn::Path::from(syn::PathSegment::from(self.subst.clone()));
                    *node = syn::Expr::Path(syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path,
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::{quote, ToTokens};

    macro_rules! test_replace_ident_in_expr {
        (
            $test_name:ident,
            $orig:expr,
            $subst:expr,
            $target:expr,
            $expected:expr,
        ) => {
            #[test]
            fn $test_name() -> Result<(), syn::Error> {
                let orig = syn::parse2::<syn::Ident>($orig).unwrap();
                let subst = syn::parse2::<syn::Ident>($subst).unwrap();
                let target = syn::parse2::<syn::Expr>($target).unwrap();
                let expected = syn::parse2::<syn::Expr>($expected).unwrap();

                let got = replace_ident_in_expr(orig, subst, target.clone());
                assert_eq!(
                    got,
                    expected,
                    "\n    got      = {},\n    expected = {}",
                    got.to_token_stream(),
                    expected.to_token_stream(),
                );

                Ok(())
            }
        };
    }

    test_replace_ident_in_expr! {
        ref_0,
        quote! { x },
        quote! { y },
        quote! { &x.0 },
        quote! { &y.0 },
    }

    test_replace_ident_in_expr! {
        dont_replace_not_is_ident,
        quote! { x },
        quote! { y },
        quote! { x::x(&x.0) },
        quote! { x::x(&y.0) },
    }
}
