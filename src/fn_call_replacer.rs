use syn::visit_mut::VisitMut;

/// Replace function calls `orig_func(receiver)` in `target` with
/// `subst_func(receiver, subst_non_receiver_args)`.
pub(crate) fn replace_fn_call_in_expr(
    orig_func: syn::Ident,
    subst_func: syn::Path,
    subst_non_receiver_args: syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
    mut target: syn::Expr,
) -> syn::Expr {
    let orig_func = syn::Path::from(syn::PathSegment::from(orig_func));
    let orig_func = syn::ExprPath {
        attrs: vec![],
        qself: None,
        path: orig_func,
    }
    .into();
    let subst_func = syn::ExprPath {
        attrs: vec![],
        qself: None,
        path: subst_func,
    }
    .into();
    let mut visitor = Visitor {
        orig_func,
        subst_func,
        subst_non_receiver_args,
    };
    visitor.visit_expr_mut(&mut target);
    target
}

struct Visitor {
    orig_func: syn::Expr,
    subst_func: syn::Expr,
    subst_non_receiver_args: syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
}

impl VisitMut for Visitor {
    fn visit_expr_mut(&mut self, node: &mut syn::Expr) {
        syn::visit_mut::visit_expr_mut(self, node);

        #[allow(clippy::single_match)]
        match node {
            syn::Expr::Call(expr) => {
                if *expr.func == self.orig_func {
                    *expr.func = self.subst_func.clone();
                    expr.args
                        .extend(self.subst_non_receiver_args.iter().cloned());
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
    use syn::parse_quote;

    macro_rules! test_replace_fn_call_in_expr {
        (
            $test_name:ident,
            $orig_func:expr,
            $subst_func:expr,
            $subst_non_receiver_args:expr,
            $target:expr,
            $expected:expr,
        ) => {
            #[test]
            fn $test_name() -> Result<(), syn::Error> {
                let orig_func = syn::parse2::<syn::Ident>($orig_func).unwrap();
                let subst_func = syn::parse2::<syn::Path>($subst_func).unwrap();
                let subst_non_receiver_args = $subst_non_receiver_args;
                let subst_non_receiver_args = parse_quote! { #subst_non_receiver_args };
                let target = syn::parse2::<syn::Expr>($target).unwrap();
                let expected = syn::parse2::<syn::Expr>($expected).unwrap();

                let got = replace_fn_call_in_expr(
                    orig_func,
                    subst_func,
                    subst_non_receiver_args,
                    target.clone(),
                );
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

    test_replace_fn_call_in_expr! {
        no_args,
        quote! { f },
        quote! { Hello::hello },
        quote! {},
        quote! { f(self.key()) },
        quote! { Hello::hello(self.key()) },
    }

    test_replace_fn_call_in_expr! {
        with_args,
        quote! { f },
        quote! { Hello::hello },
        quote! { a, b },
        quote! { f(self.key()) },
        quote! { Hello::hello(self.key(), a, b) },
    }

    test_replace_fn_call_in_expr! {
        multiple_times,
        quote! { f },
        quote! { Hello::hello },
        quote! { a },
        quote! {
            match self {
                Self::A(s) => f(&format!("{s}{s}")),
                Self::B(c) => f(c),
            }
        },
        quote! {
            match self {
                Self::A(s) => Hello::hello(&format!("{s}{s}"), a),
                Self::B(c) => Hello::hello(c, a),
            }
        },
    }

    // It doesn't care about name conflict.
    //
    // See also tests/ui/pass_scheme_name_conflict.rs
    test_replace_fn_call_in_expr! {
        name_conflict,
        quote! { f },
        quote! { Hello::hello },
        quote! { a },
        quote! {
            match self {
                Self::A(a) => f(&format!("{a}{a}")),
                Self::B(b) => f(b),
            }
        },
        quote! {
            match self {
                Self::A(a) => Hello::hello(&format!("{a}{a}"), a),
                Self::B(b) => Hello::hello(b, a),
            }
        },
    }
}
