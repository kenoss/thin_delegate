mod attr_remover;
mod delegate_to_arg;
#[cfg(not(feature = "unstable_delegate_to"))]
mod delegate_to_checker;
mod delegate_to_remover;
mod derive_delegate_args;
mod generic_param_replacer;
mod ident_replacer;
mod punctuated_parser;
mod self_replacer;

use crate::delegate_to_arg::DelegateToArg;
use crate::derive_delegate_args::DeriveDelegateArgs;
use crate::generic_param_replacer::GenericParamReplacer;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::ops::Deref;
use syn::parse_quote;
use syn::spanned::Spanned;

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

#[derive(Debug)]
pub(crate) struct TraitData {
    trait_path: syn::Path,
    generics: syn::Generics,
    sigs: Vec<syn::Signature>,
}

impl TraitData {
    pub fn new(trait_: &syn::ItemTrait, mut trait_path: syn::Path) -> Self {
        trait_path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

        let sigs = trait_
            .items
            .iter()
            .filter_map(|x| {
                let syn::TraitItem::Fn(fn_) = x else {
                    return None;
                };

                Some(fn_.sig.clone())
            })
            .collect();

        TraitData {
            trait_path,
            generics: trait_.generics.clone(),
            sigs,
        }
    }

    fn fn_ingredients(&self) -> impl Iterator<Item = FnIngredient<'_>> {
        self.sigs.iter().map(|sig| FnIngredient {
            trait_path: &self.trait_path,
            sig,
        })
    }

    fn validate(&self) -> syn::Result<()> {
        for fn_ingredient in self.fn_ingredients() {
            fn_ingredient.validate()?;
        }

        Ok(())
    }
}

struct FnIngredient<'a> {
    trait_path: &'a syn::Path,
    sig: &'a syn::Signature,
}

impl<'a> FnIngredient<'a> {
    pub fn validate(&self) -> syn::Result<()> {
        self.receiver_prefix()?;

        Ok(())
    }

    pub fn receiver_prefix(&self) -> syn::Result<TokenStream> {
        if self.sig.inputs.is_empty() {
            return Err(syn::Error::new_spanned(
                &self.sig.inputs,
                "method must have arguments.",
            ));
        }

        let syn::FnArg::Receiver(r) = &self.sig.inputs[0] else {
            return Err(syn::Error::new_spanned(
                &self.sig.inputs[0],
                "method must have receiver",
            ));
        };

        let ret = match (&r.reference, &r.mutability) {
            (Some(_), Some(_)) => quote! { &mut },
            (Some(_), None) => quote! { & },
            (None, Some(_)) => quote! {},
            (None, None) => quote! {},
        };
        Ok(ret)
    }

    pub fn args(&self) -> Vec<&syn::PatIdent> {
        self.sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(pat_type) => {
                    let syn::Pat::Ident(ident) = pat_type.pat.as_ref() else {
                        panic!("Pat should be an ident in function declaration position.");
                    };
                    Some(ident)
                }
            })
            .collect()
    }
}

#[proc_macro_attribute]
pub fn external_trait_def(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item: TokenStream = item.into();
    match external_trait_def_aux(args.into(), item.clone()) {
        Ok(x) => x.into(),
        Err(e) => TokenStream::from_iter([e.into_compile_error(), item]).into(),
    }
}

fn external_trait_def_aux(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !args.is_empty() {
        return Err(syn::Error::new_spanned(args, "arguments must be empty"));
    }

    let e = syn::Error::new(item.span(), "expected `mod ... { ... }`");
    let item = syn::parse2::<syn::Item>(item).map_err(|_| e.clone())?;
    let syn::Item::Mod(mut mod_) = item else {
        return Err(e);
    };
    let Some(ref mut content) = mod_.content else {
        return Err(e);
    };

    for item in &mut content.1 {
        #[allow(clippy::single_match)]
        match item {
            syn::Item::Trait(ref mut trait_) => {
                let attr = parse_quote! {
                    #[::thin_delegate::internal_is_external_marker]
                };
                trait_.attrs.push(attr);
            }
            _ => {}
        }
    }

    Ok(quote! { #mod_ })
}

#[proc_macro_attribute]
pub fn internal_is_external_marker(
    _args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item: TokenStream = item.into();
    syn::Error::new_spanned(item, "#[thin_delegate::register] missing for trait")
        .into_compile_error()
        .into()
}

#[proc_macro_attribute]
pub fn register(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item: TokenStream = item.into();
    match register_aux(args.into(), item.clone()) {
        Ok(x) => x.into(),
        Err(e) => TokenStream::from_iter([e.into_compile_error(), item]).into(),
    }
}

fn register_aux(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !args.is_empty() {
        return Err(syn::Error::new_spanned(args, "arguments must be empty"));
    }

    let mut item = syn::parse2::<syn::Item>(item.clone()).map_err(|_| {
        syn::Error::new(
            item.span(),
            "expected `trait ...` or `struct ...` or `enum ...`",
        )
    })?;
    let is_external = match &item {
        syn::Item::Trait(trait_) => {
            let internal_is_external_marker: syn::Attribute = parse_quote! {
                #[::thin_delegate::internal_is_external_marker]
            };
            trait_
                .attrs
                .iter()
                .any(|attr| *attr == internal_is_external_marker)
        }
        _ => false,
    };
    let macro_def = match &item {
        syn::Item::Trait(trait_) => {
            let trait_path = syn::Path::from(syn::PathSegment::from(trait_.ident.clone()));
            // Note that `trait_path` here is a kind of dummy. It's just used for creating `TraitData`.
            let trait_data = TraitData::new(trait_, trait_path);
            trait_data.validate()?;

            let feed_trait_def_for =
                macro_name_feed_trait_def_for(&trait_.ident, trait_.ident.span(), is_external);
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
        syn::Item::Struct(structenum) => {
            let feed_structenum_def_for =
                macro_name_feed_structenum_def_for(&structenum.ident, structenum.ident.span());
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
        syn::Item::Enum(structenum) => {
            let feed_structenum_def_for =
                macro_name_feed_structenum_def_for(&structenum.ident, structenum.ident.span());
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
        _ => {
            return Err(syn::Error::new_spanned(
                item,
                "expected `trait ...` or `struct ...` or `enum ...`",
            ));
        }
    };

    attr_remover::relplace_attr_with_do_nothing_in_item(
        parse_quote! { ::thin_delegate::internal_is_external_marker },
        &mut item,
    );

    #[cfg(not(feature = "unstable_delegate_to"))]
    {
        delegate_to_checker::check_non_existence(&mut item)?;
    }
    delegate_to_remover::remove_delegate_to(&mut item);

    if is_external {
        Ok(quote! {
            #macro_def
        })
    } else {
        Ok(quote! {
            #item

            #macro_def
        })
    }
}

#[proc_macro_attribute]
pub fn derive_delegate(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item: TokenStream = item.into();

    match derive_delegate_aux(args.into(), item.clone()) {
        Ok(x) => x.into(),
        Err(e) => TokenStream::from_iter([e.into_compile_error(), item]).into(),
    }
}

fn derive_delegate_aux(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let args = syn::parse2::<DeriveDelegateArgs>(args)?;
    let external_trait_def = args.external_trait_def;

    let e = syn::Error::new_spanned(&item, "expected `impl <Trait> for <Type>`");
    let item = syn::parse2::<syn::Item>(item).map_err(|_| e.clone())?;
    let syn::Item::Impl(impl_) = item else {
        return Err(e);
    };
    let Some((_, trait_path, _)) = &impl_.trait_ else {
        return Err(e);
    };
    let syn::Type::Path(structenum_path) = impl_.self_ty.deref() else {
        return Err(e);
    };

    let trait_ident = &trait_path.segments.last().unwrap().ident;
    let structenum_ident = &structenum_path.path.segments.last().unwrap().ident;

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
    Ok(quote! {
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
    })
}

#[proc_macro_attribute]
pub fn internal_derive_delegate(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match internal_derive_delegate_aux(args.into(), input.into()) {
        Ok(x) => x.into(),
        Err(e) => TokenStream::from_iter([e.into_compile_error()]).into(),
    }
}

fn internal_derive_delegate_aux(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !args.is_empty() {
        panic!();
    }

    let item = syn::parse2::<syn::Item>(item.clone()).unwrap();
    let syn::Item::Mod(mod_) = item else {
        panic!();
    };
    let content = &mod_.content.as_ref().unwrap().1;
    assert_eq!(content.len(), 3);
    let syn::Item::Trait(trait_) = &content[0] else {
        panic!();
    };
    let structenum = &content[1];
    let syn::Item::Impl(impl_) = &content[2] else {
        panic!();
    };
    // TODO: Support exceptional methods.
    assert!(impl_.items.is_empty());
    let Some((_, trait_path, _)) = &impl_.trait_ else {
        panic!()
    };

    let trait_data = TraitData::new(trait_, trait_path.clone());

    let generic_param_replacer = GenericParamReplacer::new(
        &trait_data.generics,
        &trait_path.segments.last().unwrap().arguments,
    )?;

    let funcs = trait_data
        .fn_ingredients()
        .map(|fn_ingredient| gen_impl_fn(&generic_param_replacer, structenum, fn_ingredient))
        .collect::<syn::Result<Vec<_>>>()?;

    let item_name = match &structenum {
        syn::Item::Enum(enum_) => {
            let ident = &enum_.ident;
            let generics = &enum_.generics;
            quote! { #ident #generics }
        }
        syn::Item::Struct(struct_) => {
            let ident = &struct_.ident;
            let generics = &struct_.generics;
            quote! { #ident #generics }
        }
        _ => {
            return Err(syn::Error::new(
                structenum.span(),
                "expected `enum ...` or `struct ...`",
            ));
        }
    };

    Ok(quote! {
        impl #trait_path for #item_name {
            #(#funcs)*
        }
    })
}

fn gen_impl_fn(
    generic_param_replacer: &GenericParamReplacer,
    item: &syn::Item,
    fn_ingredient: FnIngredient<'_>,
) -> syn::Result<TokenStream> {
    match item {
        syn::Item::Enum(enum_) => gen_impl_fn_enum(generic_param_replacer, enum_, fn_ingredient),
        syn::Item::Struct(struct_) => {
            gen_impl_fn_struct(generic_param_replacer, struct_, fn_ingredient)
        }
        _ => Err(syn::Error::new(
            item.span(),
            "expected `enum ...` or `struct ...`",
        )),
    }
}

fn gen_impl_fn_enum(
    generic_param_replacer: &GenericParamReplacer,
    enum_: &syn::ItemEnum,
    fn_ingredient: FnIngredient<'_>,
) -> syn::Result<TokenStream> {
    let trait_path = &fn_ingredient.trait_path;
    let method_ident = &fn_ingredient.sig.ident;
    let args = fn_ingredient.args();
    let match_arms = enum_
        .variants
        .iter()
        .map(|variant| {
            // Note that we'll remove `#[delegate_to(...)]` attribute by `delegate_to_remover::remove_delegate_to()`.
            let mut delegate_to_arg = None;
            for attr in &variant.attrs {
                match &attr.meta {
                    syn::Meta::List(meta_list) if meta_list.path.is_ident("delegate_to") => {
                        if delegate_to_arg.is_some() {
                            return Err(syn::Error::new_spanned(
                                attr,
                                "#[delegate_to(...)] can appear at most once",
                            ));
                        }

                        delegate_to_arg = Some(syn::parse2::<DelegateToArg>(meta_list.tokens.clone())?);
                    }
                    _ => {},
                }
            }

            let variant_ident = &variant.ident;
            match &variant.fields {
                syn::Fields::Named(fields) => {
                    if fields.named.len() != 1 {
                        return Err(syn::Error::new_spanned(
                            &variant.fields,
                            "fields of enum variant must be a field",
                        ));
                    }

                    let ident = fields.named[0].ident.as_ref().unwrap();
                    let receiver = if let Some(delegate_to_arg) = delegate_to_arg {
                        ident_replacer::replace_ident_in_expr(delegate_to_arg.ident, ident.clone(), delegate_to_arg.expr).to_token_stream()
                    } else {
                        ident.to_token_stream()
                    };

                    Ok(quote! {
                        Self::#variant_ident { #ident } => #trait_path::#method_ident(#receiver #(,#args)*)
                    })
                }
                syn::Fields::Unnamed(fields) => {
                    if fields.unnamed.len() != 1 {
                        return Err(syn::Error::new_spanned(
                            &variant.fields,
                            "fields of enum variant must be a field",
                        ));
                    }

                    let ident = syn::Ident::new("x", Span::call_site());
                    let receiver = if let Some(delegate_to_arg) = delegate_to_arg {
                        ident_replacer::replace_ident_in_expr(delegate_to_arg.ident, ident.clone(), delegate_to_arg.expr).to_token_stream()
                    } else {
                        ident.to_token_stream()
                    };
                    Ok(quote! {
                        Self::#variant_ident(x) => #trait_path::#method_ident(#receiver #(,#args)*)
                    })
                }
                syn::Fields::Unit => {
                    Err(syn::Error::new_spanned(
                        variant,
                        "fields of enum variant must be a field",
                    ))
                }
            }
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let sig = generic_param_replacer.replace_signature(fn_ingredient.sig.clone());
    let sig = self_replacer::make_self_hygienic_in_signature(sig);
    Ok(quote! {
        #sig {
            match self {
                #(#match_arms,)*
            }
        }
    })
}

fn gen_impl_fn_struct(
    generic_param_replacer: &GenericParamReplacer,
    struct_: &syn::ItemStruct,
    fn_ingredient: FnIngredient<'_>,
) -> syn::Result<TokenStream> {
    let field_ident = {
        if struct_.fields.len() != 1 {
            return Err(syn::Error::new(
                Span::call_site(),
                "struct must have exact one field.",
            ));
        }

        match &struct_.fields.iter().next().unwrap().ident {
            Some(ident) => quote! { #ident },
            None => quote! { 0 },
        }
    };
    let receiver_prefix = fn_ingredient.receiver_prefix().unwrap();
    let receiver = quote! { #receiver_prefix self.#field_ident };

    let sig = generic_param_replacer.replace_signature(fn_ingredient.sig.clone());
    let sig = self_replacer::make_self_hygienic_in_signature(sig);
    let trait_path = &fn_ingredient.trait_path;
    let method_ident = &fn_ingredient.sig.ident;
    let args = fn_ingredient.args();
    Ok(quote! {
        #sig {
            #trait_path::#method_ident(#receiver #(,#args)*)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! compare_result {
        ($got:expr, $expected:expr) => {
            let expected: syn::Result<TokenStream> = $expected;
            assert_eq!(
                ($got).map(|x| x.to_string()).map_err(|e| e.to_string()),
                expected.map(|x| x.to_string()).map_err(|e| e.to_string())
            );
        };
    }

    macro_rules! test_internal_derive_delegate {
        (
            $test_name:ident,
            $input:expr,
            $expected:expr,
        ) => {
            #[test]
            fn $test_name() -> syn::Result<()> {
                let args = TokenStream::new();
                let input: TokenStream = $input;
                let expected: TokenStream = $expected;

                let input = quote! {
                    mod __thin_delegate__test_mod {
                        #input
                    }
                };
                compare_result!(internal_derive_delegate_aux(args, input), Ok(expected));

                Ok(())
            }
        };
    }

    test_internal_derive_delegate! {
        r#enum,
        quote! {
            trait Hello {
                fn hello(&self) -> String;
            }

            enum Hoge {
                Named {
                    named: String,
                },
                Unnamed(char),
            }

            impl Hello for Hoge {}
        },
        quote! {
            impl Hello for Hoge {
                fn hello(&self) -> String {
                    match self {
                        Self::Named { named } => Hello::hello(named),
                        Self::Unnamed(x) => Hello::hello(x),
                    }
                }
            }
        },
    }

    test_internal_derive_delegate! {
        enum_ref_mut_receiver,
        quote! {
            pub trait Hello {
                fn hello(&mut self) -> String;
            }

            enum Hoge {
                Named {
                    named: String,
                },
                Unnamed(char),
            }

            impl Hello for Hoge {}
        },
        quote! {
            impl Hello for Hoge {
                fn hello(&mut self) -> String {
                    match self {
                        Self::Named { named } => Hello::hello(named),
                        Self::Unnamed(x) => Hello::hello(x),
                    }
                }
            }
        },
    }

    test_internal_derive_delegate! {
        enum_consume_receiver,
        quote! {
            trait Hello {
                fn hello(self) -> String;
            }

            enum Hoge {
                Named {
                    named: String,
                },
                Unnamed(char),
            }

            impl Hello for Hoge {}
        },
        quote! {
            impl Hello for Hoge {
                fn hello(self) -> String {
                    match self {
                        Self::Named { named } => Hello::hello(named),
                        Self::Unnamed(x) => Hello::hello(x),
                    }
                }
            }
        },
    }

    test_internal_derive_delegate! {
        struct_with_named_field,
        quote! {
            trait Hello {
                fn hello(&self) -> String;
            }

            struct Hoge {
                s: String,
            }

            impl Hello for Hoge {}
        },
        quote! {
            impl Hello for Hoge {
                fn hello(&self) -> String {
                    Hello::hello(&self.s)
                }
            }
        },
    }

    test_internal_derive_delegate! {
        struct_with_unnamed_field,
        quote! {
            trait Hello {
                fn hello(&self) -> String;
            }

            struct Hoge(String);

            impl Hello for Hoge {}
        },
        quote! {
            impl Hello for Hoge {
                fn hello(&self) -> String {
                    Hello::hello(&self.0)
                }
            }
        },
    }

    test_internal_derive_delegate! {
        struct_ref_mut_receiver,
        quote! {
            trait Hello {
                fn hello(&mut self) -> String;
            }

            struct Hoge {
                s: String,
            }

            impl Hello for Hoge {}
        },
        quote! {
            impl Hello for Hoge {
                fn hello(&mut self) -> String {
                    Hello::hello(&mut self.s)
                }
            }
        },
    }

    test_internal_derive_delegate! {
        struct_consume_receiver,
        quote! {
            trait Hello {
                fn hello(self) -> String;
            }

            struct Hoge {
                s: String,
            }

            impl Hello for Hoge {}
        },
        quote! {
            impl Hello for Hoge {
                fn hello(self) -> String {
                    Hello::hello(self.s)
                }
            }
        },
    }

    test_internal_derive_delegate! {
        method_with_args,
        quote! {
            trait Hello {
                fn hello(&self, prefix: &str) -> String;
            }

            enum Hoge {
                A(String),
                B(char),
            }

            impl Hello for Hoge {}
        },
        quote! {
            impl Hello for Hoge {
                fn hello(&self, prefix: &str) -> String {
                    match self {
                        Self::A(x) => Hello::hello(x, prefix),
                        Self::B(x) => Hello::hello(x, prefix),
                    }
                }
            }
        },
    }

    test_internal_derive_delegate! {
        super_trait,
        quote! {
            trait Hello: ToString {
                fn hello(&self) -> String;
            }

            enum Hoge {
                A(String),
                B(char),
            }

            impl Hello for Hoge {}
        },
        quote! {
            impl Hello for Hoge {
                fn hello(&self) -> String {
                    match self {
                        Self::A(x) => Hello::hello(x),
                        Self::B(x) => Hello::hello(x),
                    }
                }
            }
        },
    }

    test_internal_derive_delegate! {
        generics_enum,
        quote! {
            pub trait AsRef<T: ?Sized> {
                /// Converts this type into a shared reference of the (usually inferred) input type.
                #[stable(feature = "rust1", since = "1.0.0")]
                fn as_ref(&self) -> &T;
            }

            enum Hoge {
                A(String),
                B(char),
            }

            impl AsRef<str> for Hoge {}
        },
        quote! {
            impl AsRef<str> for Hoge {
                fn as_ref(&self) -> &str {
                    match self {
                        Self::A(x) => AsRef::as_ref(x),
                        Self::B(x) => AsRef::as_ref(x),
                    }
                }
            }
        },
    }

    test_internal_derive_delegate! {
        generics_struct,
        quote! {
            pub trait AsRef<T: ?Sized> {
                /// Converts this type into a shared reference of the (usually inferred) input type.
                #[stable(feature = "rust1", since = "1.0.0")]
                fn as_ref(&self) -> &T;
            }

            struct Hoge {
                s: String,
            }

            impl AsRef<str> for Hoge {}
        },
        quote! {
            impl AsRef<str> for Hoge {
                fn as_ref(&self) -> &str {
                    AsRef::as_ref(&self.s)
                }
            }
        },
    }

    test_internal_derive_delegate! {
        generics_specilize_complex,
        quote! {
            pub trait AsRef<T: ?Sized> {
                /// Converts this type into a shared reference of the (usually inferred) input type.
                #[stable(feature = "rust1", since = "1.0.0")]
                fn as_ref(&self) -> &T;
            }

            struct Hoge(Box<dyn Fn(usize) -> usize>);

            impl AsRef<(dyn Fn(usize) -> usize + 'static)> for Hoge {}
        },
        quote! {
            impl AsRef<(dyn Fn(usize) -> usize + 'static)> for Hoge {
                fn as_ref(&self) -> &(dyn Fn(usize) -> usize + 'static) {
                    AsRef::as_ref(&self.0)
                }
            }
        },
    }

    test_internal_derive_delegate! {
        generics_specilize_lifetime,
        quote! {
            pub trait Hello<'a, T> {
                fn hello(&self) -> &'a T;
            }

            struct Hoge<'p>(&'p str);

            impl Hello<'p, str> for Hoge<'p> {}
        },
        quote! {
            impl Hello<'p, str> for Hoge<'p> {
                fn hello(&self) -> &'p str {
                    Hello::hello(&self.0)
                }
            }
        },
    }

    test_internal_derive_delegate! {
        custom_receiver,
        quote! {
            pub trait AsRef<T: ?Sized> {
                /// Converts this type into a shared reference of the (usually inferred) input type.
                #[stable(feature = "rust1", since = "1.0.0")]
                fn as_ref(&self) -> &T;
            }

            enum Hoge {
                #[delegate_to(x => &x.0)]
                A((String, u8)),
            }

            impl AsRef<str> for Hoge {}
        },
        quote! {
            impl AsRef<str> for Hoge {
                fn as_ref(&self) -> &str {
                    match self {
                        Self::A(x) => AsRef::as_ref(&x.0),
                    }
                }
            }
        },
    }
}
