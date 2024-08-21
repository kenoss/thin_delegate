mod storage;

use indoc::indoc;
use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse_macro_input;
use syn::spanned::Spanned;

#[derive(Debug)]
pub(crate) struct FnIngredient {
    trait_name: syn::Path,
    ident: syn::Ident,
    sig: syn::Signature,
}

#[derive(Debug)]
pub(crate) struct StorableFnIngredient {
    trait_name: String,
    ident: String,
    sig: String,
}

impl From<&FnIngredient> for StorableFnIngredient {
    fn from(x: &FnIngredient) -> Self {
        Self {
            trait_name: x.trait_name.to_token_stream().to_string(),
            ident: x.ident.to_token_stream().to_string(),
            sig: x.sig.to_token_stream().to_string(),
        }
    }
}

impl From<&StorableFnIngredient> for FnIngredient {
    fn from(x: &StorableFnIngredient) -> Self {
        Self {
            trait_name: syn::parse2::<syn::Path>(x.trait_name.parse().unwrap()).unwrap(),
            ident: syn::parse2::<syn::Ident>(x.ident.parse().unwrap()).unwrap(),
            sig: syn::parse2::<syn::Signature>(x.sig.parse().unwrap()).unwrap(),
        }
    }
}

#[proc_macro_attribute]
pub fn register(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as syn::Item);

    match register_aux(args.into(), &item) {
        Ok(x) => x.into(),
        Err(e) => TokenStream::from_iter([e.into_compile_error(), (quote! { #item })]).into(),
    }
}

fn register_aux(args: TokenStream, item: &syn::Item) -> syn::Result<TokenStream> {
    if args.is_empty() {
        return Err(syn::Error::new_spanned(
            args,
            "arguments must not be empty: `#[thin_delegate::register(Type)]`",
        ));
    }

    let path = syn::parse2::<syn::Path>(args.clone()).map_err(|_| {
        syn::Error::new_spanned(
            args,
            "type argument expected: `#[thin_delegate::register(Type)]`",
        )
    })?;

    let syn::Item::Trait(trait_) = item else {
        return Err(syn::Error::new(item.span(), "expected `trait ...`"));
    };

    let fn_ingredients = trait_
        .items
        .iter()
        .filter_map(|x| {
            let syn::TraitItem::Fn(fn_) = x else {
                return None;
            };

            let fn_ingredient = FnIngredient {
                trait_name: path.clone(),
                ident: fn_.sig.ident.clone(),
                sig: fn_.sig.clone(),
            };

            Some(fn_ingredient)
        })
        .collect_vec();

    storage::store(&path, &fn_ingredients)?;

    // TODO: Split `register()` and `register_temporarily()` and return tokens for the former.
    Ok(TokenStream::new())
}

#[proc_macro_attribute]
pub fn derive_delegate(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as syn::Item);

    match derive_delegate_aux(args.into(), &item) {
        Ok(x) => x.into(),
        Err(e) => TokenStream::from_iter([e.into_compile_error(), (quote! { #item })]).into(),
    }
}

fn derive_delegate_aux(args: TokenStream, item: &syn::Item) -> syn::Result<TokenStream> {
    if args.is_empty() {
        return Err(syn::Error::new_spanned(
            args,
            "arguments must not be empty `#[thin_delegate::derive_delegate(Type)]`",
        ));
    }

    let path = syn::parse2::<syn::Path>(args.clone()).map_err(|_| {
        syn::Error::new_spanned(
            args,
            "type argument expected: `#[thin_delegate::derive_delegate(Type)]`",
        )
    })?;

    let Some(fn_ingredients) = storage::get(&path) else {
        return Err(syn::Error::new(
            Span::call_site(),
            format!(
                indoc! {r#"
                    trait not registered: path = {path}

                    hint: Add `#[thin_delegate::register({path})]` for trait `{path}`
                "#},
                path = path.to_token_stream(),
            ),
        ));
    };

    let funcs = fn_ingredients
        .iter()
        .map(|fn_ingredient| gen_impl_fn(fn_ingredient, item))
        .collect::<syn::Result<Vec<_>>>()?;

    let syn::Item::Enum(enum_) = item else {
        return Err(syn::Error::new(item.span(), "expected `enum ...`"));
    };

    let item_ident = &enum_.ident;
    Ok(quote! {
        #item

        impl #path for #item_ident {
            #(#funcs)*
        }
    })
}

fn gen_impl_fn(fn_ingredient: &FnIngredient, item: &syn::Item) -> syn::Result<TokenStream> {
    let syn::Item::Enum(enum_) = item else {
        return Err(syn::Error::new(item.span(), "expected `enum ...`"));
    };

    let args = fn_ingredient
        .sig
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
        .collect_vec();

    let trait_name = &fn_ingredient.trait_name;
    let method_ident = &fn_ingredient.ident;
    let match_arms = enum_
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            match &variant.fields {
                syn::Fields::Named(_) => {
                    todo!();
                }
                syn::Fields::Unnamed(fields) => {
                    if fields.unnamed.len() != 1 {
                        todo!();
                    }

                    quote! {
                        Self::#variant_ident(x) => #trait_name::#method_ident(x #(,#args)*)
                    }
                }
                syn::Fields::Unit => {
                    todo!();
                }
            }
        })
        .collect_vec();

    let sig = &fn_ingredient.sig;
    Ok(quote! {
        #sig {
            match self {
                #(#match_arms,)*
            }
        }
    })
}
