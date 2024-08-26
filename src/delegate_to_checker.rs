use syn::visit_mut::VisitMut;

pub(crate) fn check_non_existence(item: &mut syn::Item) -> syn::Result<()> {
    let mut visitor = Visitor { error: None };
    visitor.visit_item_mut(item);
    match visitor.error {
        None => Ok(()),
        Some(e) => Err(e),
    }
}

struct Visitor {
    error: Option<syn::Error>,
}

// Use `visit_*_mut()` as we may need to change enum variant when it matches.
impl VisitMut for Visitor {
    fn visit_attribute_mut(&mut self, node: &mut syn::Attribute) {
        syn::visit_mut::visit_attribute_mut(self, node);

        #[allow(clippy::single_match)]
        match &node.meta {
            syn::Meta::List(meta_list) if meta_list.path.is_ident("delegate_to") => {
                self.error.get_or_insert_with(|| {
                    syn::Error::new_spanned(
                        node,
                        "#[delegate_to(...)] requires feature flag `unstable_delegate_to`",
                    )
                });
            }
            _ => {}
        }
    }
}
