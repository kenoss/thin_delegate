use indoc::indoc;
use itertools::izip;
use quote::ToTokens;
use std::collections::HashMap;
use syn::visit_mut::VisitMut;

pub(crate) struct GenericParamReplacer {
    lifetimes: HashMap<syn::Lifetime, syn::Lifetime>,
    types: HashMap<syn::TypePath, syn::Type>,
}

impl GenericParamReplacer {
    pub fn new(orig: &syn::Path, subst: &syn::Path) -> syn::Result<Self> {
        let mut orig_ = orig.clone();
        orig_.segments.last_mut().unwrap().arguments = syn::PathArguments::None;
        let mut subst_ = orig.clone();
        subst_.segments.last_mut().unwrap().arguments = syn::PathArguments::None;
        if orig_ != subst_ {
            panic!("precondition. it's ensured as orig comes from storage.");
        }

        let mut this = Self {
            lifetimes: HashMap::new(),
            types: HashMap::new(),
        };

        let (orig, subst) = match (
            &orig.segments.last().unwrap().arguments,
            &subst.segments.last().unwrap().arguments,
        ) {
            (syn::PathArguments::None, syn::PathArguments::None) => {
                return Ok(this);
            }
            (syn::PathArguments::None, _) | (_, syn::PathArguments::None) => {
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
            (
                syn::PathArguments::AngleBracketed(orig),
                syn::PathArguments::AngleBracketed(subst),
            ) => (orig, subst),
            _ => {
                panic!("The author of crate believes that this case doesn't happen. Please file a bag.");
            }
        };
        if orig.args.len() != subst.args.len() {
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

        for (o, s) in izip!(orig.args.iter(), subst.args.iter()) {
            match o {
                syn::GenericArgument::Lifetime(o) => {
                    let syn::GenericArgument::Lifetime(s) = s else {
                        todo!();
                    };
                    this.lifetimes.insert(o.clone(), s.clone());
                }
                syn::GenericArgument::Type(syn::Type::Path(o)) => {
                    let syn::GenericArgument::Type(s) = s else {
                        todo!();
                    };
                    this.types.insert(o.clone(), s.clone());
                }
                _ => {
                    todo!();
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

impl VisitMut for Visitor<'_> {
    fn visit_lifetime_mut(&mut self, node: &mut syn::Lifetime) {
        if let Some(subst) = self.0.lifetimes.get(node) {
            *node = subst.clone();
        }
    }

    // Use `visit_type_mut()` as we need to change enum variant when it matches.
    fn visit_type_mut(&mut self, node: &mut syn::Type) {
        match node {
            syn::Type::Array(x) => {
                self.visit_type_array_mut(x);
            }
            syn::Type::BareFn(x) => {
                self.visit_type_bare_fn_mut(x);
            }
            syn::Type::Group(x) => {
                self.visit_type_group_mut(x);
            }
            syn::Type::ImplTrait(x) => {
                self.visit_type_impl_trait_mut(x);
            }
            syn::Type::Infer(x) => {
                self.visit_type_infer_mut(x);
            }
            syn::Type::Macro(x) => {
                self.visit_type_macro_mut(x);
            }
            syn::Type::Never(x) => {
                self.visit_type_never_mut(x);
            }
            syn::Type::Paren(x) => {
                self.visit_type_paren_mut(x);
            }
            syn::Type::Path(x) => {
                if let Some(subst) = self.0.types.get(x) {
                    *node = subst.clone();
                } else {
                    self.visit_type_path_mut(x);
                }
            }
            syn::Type::Ptr(x) => {
                self.visit_type_ptr_mut(x);
            }
            syn::Type::Reference(x) => {
                self.visit_type_reference_mut(x);
            }
            syn::Type::Slice(x) => {
                self.visit_type_slice_mut(x);
            }
            syn::Type::TraitObject(x) => {
                self.visit_type_trait_object_mut(x);
            }
            syn::Type::Tuple(x) => {
                self.visit_type_tuple_mut(x);
            }
            syn::Type::Verbatim(_x) => {
                // nop
            }
            _ => {
                unimplemented!(
                    "`syn::Type` is `non_exhaustive`. \
                     Allow compile and raise an error for new arms. \
                     Please file a bug when new ones are added."
                );
            }
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
                let orig = syn::parse2::<syn::Path>($orig).unwrap();
                let subst = syn::parse2::<syn::Path>($subst).unwrap();
                let func = syn::parse2::<syn::TraitItemFn>($func).unwrap();
                let func_replaced_expected =
                    syn::parse2::<syn::TraitItemFn>($func_replaced_expected).unwrap();

                let generic_param_replacer = GenericParamReplacer::new(&orig, &subst)?;
                assert_eq!(
                    generic_param_replacer.replace_signature(func.sig.clone()),
                    func_replaced_expected.sig
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
        quote! { fn hello(&self, x: &Vec<T>) -> String; },
        quote! { Hello<&str> },
        quote! { fn hello(&self, x: &Vec<&str>) -> String; },
    }

    test_replace_signature! {
        type_array_type,
        quote! { AsRef<T> },
        quote! { fn as_ref(&self) -> &T; },
        quote! { AsRef<[u8]> },
        quote! { fn as_ref(&self) -> &[u8]; },
    }

    test_replace_signature! {
        lifetime,
        quote! { Hello<'a, T> },
        quote! { fn hello(&self) -> &'a T; },
        quote! { Hello<'p, str> },
        quote! { fn hello(&self) -> &'p str; },
    }
}
