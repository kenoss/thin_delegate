use itertools::Itertools;
use syn::visit_mut::VisitMut;

pub(crate) fn remove_delegate_to(item: &mut syn::Item) {
    let mut visitor = Visitor;
    visitor.visit_item_mut(item);
}

struct Visitor;

// Use `visit_*_mut()` as we may need to change enum variant when it matches.
impl VisitMut for Visitor {
    fn visit_variant_mut(&mut self, node: &mut syn::Variant) {
        syn::visit_mut::visit_variant_mut(self, node);

        // TODO: Use `Vec::extract_if()` once it is stabilized.
        if let Some((i, _)) = node.attrs.iter().find_position(|attr|
            matches!(&attr.meta, syn::Meta::List(meta_list) if meta_list.path.is_ident("delegate_to"))
        ) {
            // Note that it is already checked that `#[delegate_to(...)]` appears at most once.
            node.attrs.remove(i);
        }
    }
}
