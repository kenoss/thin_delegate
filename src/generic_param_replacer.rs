use indoc::indoc;
use itertools::izip;
use quote::ToTokens;
use std::collections::HashMap;
use syn::visit_mut::VisitMut;

pub(crate) struct GenericParamReplacer {
    replace_by: HashMap<syn::TypePath, syn::Type>,
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

        let (orig, subst) = match (
            &orig.segments.last().unwrap().arguments,
            &subst.segments.last().unwrap().arguments,
        ) {
            (syn::PathArguments::None, syn::PathArguments::None) => {
                return Ok(Self {
                    replace_by: HashMap::new(),
                });
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

        let mut replace_by = HashMap::new();
        for (o, s) in izip!(orig.args.iter(), subst.args.iter()) {
            match o {
                syn::GenericArgument::Type(syn::Type::Path(o)) => {
                    let syn::GenericArgument::Type(s) = s else {
                        todo!();
                    };
                    replace_by.insert(o.clone(), s.clone());
                }
                _ => {
                    todo!();
                }
            }
        }

        Ok(Self { replace_by })
    }

    pub fn replace_signature(&self, mut sig: syn::Signature) -> syn::Signature {
        let mut visitor = Visitor {
            replace_by: &self.replace_by,
        };
        visitor.visit_signature_mut(&mut sig);
        sig
    }
}

struct Visitor<'a> {
    replace_by: &'a HashMap<syn::TypePath, syn::Type>,
}

impl VisitMut for Visitor<'_> {
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
                if let Some(subst) = self.replace_by.get(x) {
                    *node = subst.clone();
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
                unimplemented!("`syn::Type` is `non_exhaustive`. Allow compile and raise an error for new arms. Please file a bug when new ones are added.");
            }
        }
    }
}
