pub(crate) fn trim_generic_param(path: &syn::Path) -> syn::Path {
    let mut path = path.clone();
    path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;
    path
}
