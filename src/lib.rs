mod storage;

use crate::storage::Storage;
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
    let mut storage = Storage::global();

    match register_aux(&mut storage, args.into(), &item) {
        Ok(x) => x.into(),
        Err(e) => TokenStream::from_iter([e.into_compile_error(), (quote! { #item })]).into(),
    }
}

fn register_aux(
    storage: &mut Storage,
    args: TokenStream,
    item: &syn::Item,
) -> syn::Result<TokenStream> {
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

    storage.store(&path, &fn_ingredients)?;

    // TODO: Split `register()` and `register_temporarily()` and return tokens for the former.
    Ok(TokenStream::new())
}

#[proc_macro_attribute]
pub fn derive_delegate(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as syn::Item);
    let mut storage = Storage::global();

    match derive_delegate_aux(&mut storage, args.into(), &item) {
        Ok(x) => x.into(),
        Err(e) => TokenStream::from_iter([e.into_compile_error(), (quote! { #item })]).into(),
    }
}

fn derive_delegate_aux(
    storage: &mut Storage,
    args: TokenStream,
    item: &syn::Item,
) -> syn::Result<TokenStream> {
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

    let Some(fn_ingredients) = storage.get(&path) else {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::TestStorageFactory;

    macro_rules! compare_result {
        ($got:expr, $expected:expr) => {
            let expected: syn::Result<TokenStream> = $expected;
            assert_eq!(
                ($got).map(|x| x.to_string()).map_err(|e| e.to_string()),
                expected.map(|x| x.to_string()).map_err(|e| e.to_string())
            );
        };
    }

    macro_rules! test_register_derive_delegate {
        ($test_name:ident,
         $register_args:expr,
         $register_input:expr,
         $register_expected:expr,
         $derive_delegate_args:expr,
         $derive_delegate_input:expr,
         $derive_delegate_expected:expr) => {
            #[test]
            fn $test_name() -> Result<(), syn::Error> {
                let mut factory = TestStorageFactory::new();
                let mut storage = factory.factory();

                let args = $register_args;
                let input = $register_input;
                compare_result!(
                    register_aux(&mut storage, args, &syn::parse2::<syn::Item>(input)?),
                    Ok($register_expected)
                );

                let args = $derive_delegate_args;
                let input = $derive_delegate_input;
                compare_result!(
                    derive_delegate_aux(&mut storage, args, &syn::parse2::<syn::Item>(input)?),
                    Ok($derive_delegate_expected)
                );

                Ok(())
            }
        };
    }

    test_register_derive_delegate! {
        test_basic,
        // register
        quote! { ToString },
        quote! {
            pub trait ToString {
                /// Converts the given value to a `String`.
                ///
                /// # Examples
                ///
                /// ```
                /// let i = 5;
                /// let five = String::from("5");
                ///
                /// assert_eq!(five, i.to_string());
                /// ```
                #[rustc_conversion_suggestion]
                #[stable(feature = "rust1", since = "1.0.0")]
                #[cfg_attr(not(test), rustc_diagnostic_item = "to_string_method")]
                fn to_string(&self) -> String;
            }
        },
        quote! {},
        // derive_delegate
        quote! { ToString },
        quote! {
            enum Hoge {
                A(String),
                B(char),
            }
        },
        quote! {
            enum Hoge {
                A(String),
                B(char),
            }

            impl ToString for Hoge {
                fn to_string(& self) -> String {
                    match self {
                        Self::A(x) => ToString::to_string(x),
                        Self::B(x) => ToString::to_string(x),
                    }
                }
            }
        }
    }

    test_register_derive_delegate! {
        test_method_with_args,
        // register
        quote! { Hello },
        quote! {
            pub trait Hello {
                fn hello(&self, prefix: &str) -> String;
            }
        },
        quote! {},
        // derive_delegate
        quote! { Hello },
        quote! {
            enum Hoge {
                A(String),
                B(char),
            }
        },
        quote! {
            enum Hoge {
                A(String),
                B(char),
            }

            impl Hello for Hoge {
                fn hello(&self, prefix: &str) -> String {
                    match self {
                        Self::A(x) => Hello::hello(x, prefix),
                        Self::B(x) => Hello::hello(x, prefix),
                    }
                }
            }
        }
    }
}
