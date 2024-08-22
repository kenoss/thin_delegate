use indoc::indoc;
use itertools::izip;
use quote::ToTokens;
use std::collections::HashMap;
use syn::visit_mut::VisitMut;

pub(crate) struct GenericParamReplacer {
    replace_by: HashMap<syn::TypePath, syn::TypePath>,
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
            let syn::GenericArgument::Type(syn::Type::Path(o)) = o else {
                todo!();
            };
            let syn::GenericArgument::Type(syn::Type::Path(s)) = s else {
                todo!();
            };
            replace_by.insert(o.clone(), s.clone());
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
    replace_by: &'a HashMap<syn::TypePath, syn::TypePath>,
}

impl VisitMut for Visitor<'_> {
    fn visit_type_path_mut(&mut self, node: &mut syn::TypePath) {
        if let Some(subst) = self.replace_by.get(node) {
            *node = subst.clone();
        }
    }
}
