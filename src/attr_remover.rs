use syn::parse_quote;
use syn::visit_mut::VisitMut;

/// Replaces attributes that has `meta == syn::Meta::Path(path)` with "do nothing" attribute.
pub(crate) fn relplace_attr_with_do_nothing_in_item(path: syn::Path, node: &mut syn::Item) {
    let meta = syn::Meta::Path(path);
    let pred = move |attr: &syn::Attribute| attr.meta == meta;
    let mut visitor = Visitor {
        pred: Box::new(pred),
    };
    visitor.visit_item_mut(node);
}

struct Visitor {
    pred: Box<dyn Fn(&syn::Attribute) -> bool>,
}

// It's troublesome to remove attributes as `attrs: Vec<syn::Attribute>` appears a lot in the
// structures in `syn`. To avoid this, we utilize that `all()` is `true` and `#[cfg(all())]` is a
// "do nothing" attribute.
impl VisitMut for Visitor {
    fn visit_attribute_mut(&mut self, node: &mut syn::Attribute) {
        syn::visit_mut::visit_attribute_mut(self, node);

        if (self.pred)(node) {
            *node = parse_quote! { #[cfg(all())] };
        }
    }
}
