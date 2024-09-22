// FYI: See `exec_internal_derive_delegate()` first.

use proc_macro2::{Span, TokenStream};
use quote::quote;

fn macro_name_feed_trait_def_for<T>(trait_name: &T, span: Span, is_external: bool) -> syn::Ident
where
    T: std::fmt::Display,
{
    let external = if is_external { "external_" } else { "" };
    syn::Ident::new(
        &format!("__thin_delegate__feed_trait_def_for_{external}{trait_name}"),
        span,
    )
}

fn macro_name_feed_structenum_def_for<T>(structenum_name: &T, span: Span) -> syn::Ident
where
    T: std::fmt::Display,
{
    syn::Ident::new(
        &format!("__thin_delegate__feed_structenum_def_for_{structenum_name}",),
        span,
    )
}

pub(crate) fn define_macro_feed_trait_def_for(
    ident: &syn::Ident,
    span: Span,
    is_external: bool,
    trait_: &syn::ItemTrait,
) -> TokenStream {
    let feed_trait_def_for = macro_name_feed_trait_def_for(ident, span, is_external);
    quote! {
        macro_rules! #feed_trait_def_for {
            {
                @KONT { $kont:path },
                $(@$arg_key:ident { $arg_value:tt },)*
            } => {
                $kont! {
                    $(@$arg_key { $arg_value },)*
                    @TRAIT_DEF { #trait_ },
                }
            }
        }
        pub(crate) use #feed_trait_def_for;
    }
}

pub(crate) fn define_macro_feed_structenum_def_for(
    ident: &syn::Ident,
    span: Span,
    structenum: TokenStream,
) -> TokenStream {
    let feed_structenum_def_for = macro_name_feed_structenum_def_for(ident, span);
    quote! {
        macro_rules! #feed_structenum_def_for {
            {
                @KONT { $kont:path },
                $(@$arg_key:ident { $arg_value:tt },)*
            } => {
                $kont! {
                    $(@$arg_key { $arg_value },)*
                    @STRUCTENUM_DEF { #structenum },
                }
            }
        }
        pub(crate) use #feed_structenum_def_for;
    }
}

pub(crate) fn exec_internal_derive_delegate(
    trait_ident: &syn::Ident,
    structenum_ident: &syn::Ident,
    external_trait_def: &Option<syn::Path>,
    impl_: &syn::ItemImpl,
) -> TokenStream {
    let feed_trait_def_for = if let Some(external_trait_def) = &external_trait_def {
        let feed_trait_def_for =
            macro_name_feed_trait_def_for(&trait_ident, trait_ident.span(), true);
        quote! { #external_trait_def::#feed_trait_def_for }
    } else {
        let feed_trait_def_for =
            macro_name_feed_trait_def_for(&trait_ident, trait_ident.span(), false);
        quote! { #feed_trait_def_for }
    };
    let feed_structenum_def_for =
        macro_name_feed_structenum_def_for(&structenum_ident, structenum_ident.span());

    // Collect trait and structenum defs by CPS:
    //
    //    #feed_trait_def_for!
    // -> __thin_delegate__trampoline1!
    // -> #feed_structenum_def_for!
    // -> __thin_delegate__trampoline2!
    // -> #[::thin_delegate::internal_derive_delegate]
    quote! {
        macro_rules! __thin_delegate__trampoline2 {
            {
                @IMPL {{ $impl:item }},
                @TRAIT_DEF { $trait_def:item },
                @STRUCTENUM_DEF { $structenum_def:item },
            } => {
                // TODO: Add a test that uses `#[thin_delegate::derive_delegate]` twice.
                #[::thin_delegate::internal_derive_delegate]
                mod __thin_delegate__change_this_name {
                    $trait_def

                    $structenum_def

                    $impl
                }
            }
        }

        macro_rules! __thin_delegate__trampoline1 {
            {
                @IMPL {{ $impl:item }},
                @TRAIT_DEF { $trait_def:item },
            } => {
                #feed_structenum_def_for! {
                    @KONT { __thin_delegate__trampoline2 },
                    @IMPL {{ $impl }},
                    @TRAIT_DEF { $trait_def },
                }
            }
        }

        #feed_trait_def_for! {
            @KONT { __thin_delegate__trampoline1 },
            @IMPL {{ #impl_ }},
        }
    }
}
