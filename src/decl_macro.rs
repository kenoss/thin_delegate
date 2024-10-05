// FYI: See `exec_internal_derive_delegate()` first.

use proc_macro2::{Span, TokenStream};
use quote::quote;

fn macro_name_feed_trait_def_of<T>(trait_name: &T, span: Span, is_external: bool) -> syn::Ident
where
    T: std::fmt::Display,
{
    let external = if is_external { "external_" } else { "" };
    syn::Ident::new(
        &format!("__thin_delegate__feed_trait_def_of_{external}{trait_name}"),
        span,
    )
}

fn macro_name_feed_structenum_def_of<T>(structenum_name: &T, span: Span) -> syn::Ident
where
    T: std::fmt::Display,
{
    syn::Ident::new(
        &format!("__thin_delegate__feed_structenum_def_of_{structenum_name}",),
        span,
    )
}

pub(crate) fn define_macro_feed_trait_def_of(
    ident: &syn::Ident,
    span: Span,
    is_external: bool,
    trait_: &syn::ItemTrait,
) -> TokenStream {
    let feed_trait_def_of = macro_name_feed_trait_def_of(ident, span, is_external);
    quote! {
        macro_rules! #feed_trait_def_of {
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
        pub(crate) use #feed_trait_def_of;
    }
}

pub(crate) fn define_macro_feed_structenum_def_of(
    ident: &syn::Ident,
    span: Span,
    structenum: &syn::Item,
) -> TokenStream {
    let feed_structenum_def_of = macro_name_feed_structenum_def_of(ident, span);
    quote! {
        macro_rules! #feed_structenum_def_of {
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
        pub(crate) use #feed_structenum_def_of;
    }
}

pub(crate) fn exec_internal_derive_delegate(
    trait_ident: &syn::Ident,
    structenum_ident: &syn::Ident,
    external_trait_def: &Option<syn::Path>,
    args: TokenStream,
    impl_: &syn::ItemImpl,
) -> TokenStream {
    let feed_trait_def_of = if let Some(external_trait_def) = &external_trait_def {
        let feed_trait_def_of =
            macro_name_feed_trait_def_of(&trait_ident, trait_ident.span(), true);
        quote! { #external_trait_def::#feed_trait_def_of }
    } else {
        let feed_trait_def_of =
            macro_name_feed_trait_def_of(&trait_ident, trait_ident.span(), false);
        quote! { #feed_trait_def_of }
    };
    let feed_structenum_def_of =
        macro_name_feed_structenum_def_of(&structenum_ident, structenum_ident.span());

    // Collect trait and structenum defs by CPS:
    //
    //    #feed_trait_def_of!
    // -> __thin_delegate__trampoline1!
    // -> #feed_structenum_def_of!
    // -> __thin_delegate__trampoline2!
    // -> #[::thin_delegate::internal_derive_delegate]
    quote! {
        macro_rules! __thin_delegate__trampoline2 {
            {
                @IMPL {{ $impl:item }},
                @TRAIT_DEF { $trait_def:item },
                @STRUCTENUM_DEF { $structenum_def:item },
            } => {
                #[::thin_delegate::internal_derive_delegate(#args)]
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
                #feed_structenum_def_of! {
                    @KONT { __thin_delegate__trampoline2 },
                    @IMPL {{ $impl }},
                    @TRAIT_DEF { $trait_def },
                }
            }
        }

        #feed_trait_def_of! {
            @KONT { __thin_delegate__trampoline1 },
            @IMPL {{ #impl_ }},
        }
    }
}
