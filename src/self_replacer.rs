use proc_macro2::Span;
use syn::visit_mut::VisitMut;

pub(crate) fn make_self_hygienic_in_signature(mut target: syn::Signature) -> syn::Signature {
    let mut visitor = Visitor;
    visitor.visit_signature_mut(&mut target);
    target
}

/// Replaces `self` to avoid issues around macro hygienicity.
///
/// `thin_delegate` transfers definition of a trait and a struct/enum to
/// `#[thin_delegate::derive_delegate]` by using declarative macro.
/// `#[thin_delegate::__internal__derive_delegate]` processes a token stream in the macro context.
/// If we use `self` in this token stream as is, an error like the following arise:
///
/// ```text
/// error[E0424]: expected value, found module `self`
///   --> src/main.rs:24:1
///    |
/// 3  |     fn hello(&self) -> String;
///    |     -- this function has a `self` parameter, but a macro invocation can only access identifiers it receives from parameters
/// ...
/// 24 | #[thin_delegate::derive_delegate]
///    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `self` value is a keyword only available in methods with a `self` parameter
///    |
///    = note: this error originates in the attribute macro `::thin_delegate::__internal__derive_delegate` which comes from the expansion of the attribute macro `thin_delegate::derive_delegate` (in Nightly builds, run with -Z macro-backtrace for more info)
/// For more information about this error, try `rustc --explain E0424`.
/// ```
///
/// Rust's macro hygienicity forbids use of `self` in declarative macros.
/// We can resolve it by replacing `self` in the token stream by `self` generated in a proc macro,
/// which this `Visitor` does.
struct Visitor;

impl VisitMut for Visitor {
    fn visit_receiver_mut(&mut self, node: &mut syn::Receiver) {
        node.self_token = syn::Token![self](Span::call_site());
    }

    fn visit_expr_path_mut(&mut self, node: &mut syn::ExprPath) {
        if node.path.is_ident("self") {
            let ident = syn::Ident::new("self", Span::call_site());
            node.path = syn::Path::from(syn::PathSegment::from(ident));
        }
    }
}
