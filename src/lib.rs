mod generic_param_replacer;
mod punctuated_parser;
mod storage;

use crate::generic_param_replacer::GenericParamReplacer;
use crate::punctuated_parser::PunctuatedParser;
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

impl FnIngredient {
    pub fn validate(&self) -> syn::Result<()> {
        self.receiver_prefix()?;

        Ok(())
    }

    pub fn trait_name_without_generic_param(&self) -> syn::Path {
        let mut trait_name = self.trait_name.clone();
        trait_name.segments.last_mut().unwrap().arguments = syn::PathArguments::None;
        trait_name
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
    for fn_ingredient in &fn_ingredients {
        fn_ingredient.validate()?;
    }

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

    let paths = syn::parse2::<PunctuatedParser<syn::Path, syn::token::Comma>>(args)?.into_inner();

    let impls = paths
        .iter()
        .map(|path| derive_delegate_aux_1(storage, item, path))
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        #item

        #(#impls)*
    })
}

fn derive_delegate_aux_1(
    storage: &mut Storage,
    item: &syn::Item,
    path: &syn::Path,
) -> syn::Result<TokenStream> {
    let Some(fn_ingredients) = storage.get(path) else {
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

    if fn_ingredients.is_empty() {
        return Ok(quote! {});
    }

    let generic_param_replacer =
        GenericParamReplacer::new(&fn_ingredients.first().unwrap().trait_name, path)?;

    let funcs = fn_ingredients
        .iter()
        .map(|fn_ingredient| gen_impl_fn(&generic_param_replacer, item, fn_ingredient))
        .collect::<syn::Result<Vec<_>>>()?;

    let item_name = match item {
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
                item.span(),
                "expected `enum ...` or `struct ...`",
            ));
        }
    };

    Ok(quote! {
        impl #path for #item_name {
            #(#funcs)*
        }
    })
}

fn gen_impl_fn(
    generic_param_replacer: &GenericParamReplacer,
    item: &syn::Item,
    fn_ingredient: &FnIngredient,
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
    fn_ingredient: &FnIngredient,
) -> syn::Result<TokenStream> {
    let trait_name = fn_ingredient.trait_name_without_generic_param();
    let method_ident = &fn_ingredient.ident;
    let args = fn_ingredient.args();
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

    let sig = generic_param_replacer.replace_signature(fn_ingredient.sig.clone());
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
    fn_ingredient: &FnIngredient,
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
    let trait_name = fn_ingredient.trait_name_without_generic_param();
    let method_ident = &fn_ingredient.ident;
    let args = fn_ingredient.args();
    Ok(quote! {
        #sig {
            #trait_name::#method_ident(#receiver #(,#args)*)
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
        (
            $test_name:ident,
            $register_args:expr,
            $register_input:expr,
            $register_expected:expr,
            $derive_delegate_args:expr,
            $derive_delegate_input:expr,
            $derive_delegate_expected:expr,
        ) => {
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

    macro_rules! test_register_register_derive_delegate {
        (
            $test_name:ident,
            $register1_args:expr,
            $register1_input:expr,
            $register1_expected:expr,
            $register2_args:expr,
            $register2_input:expr,
            $register2_expected:expr,
            $derive_delegate_args:expr,
            $derive_delegate_input:expr,
            $derive_delegate_expected:expr,
        ) => {
            #[test]
            fn $test_name() -> Result<(), syn::Error> {
                let mut factory = TestStorageFactory::new();
                let mut storage = factory.factory();

                let args = $register1_args;
                let input = $register1_input;
                compare_result!(
                    register_aux(&mut storage, args, &syn::parse2::<syn::Item>(input)?),
                    Ok($register1_expected)
                );

                let args = $register2_args;
                let input = $register2_input;
                compare_result!(
                    register_aux(&mut storage, args, &syn::parse2::<syn::Item>(input)?),
                    Ok($register2_expected)
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

    macro_rules! test_as_ref {
        (
            $test_name:ident,
            $derive_delegate_args:expr,
            $derive_delegate_input:expr,
            $derive_delegate_expected:expr,
        ) => {
            test_register_derive_delegate! {
                $test_name,
                // register
                quote! { AsRef<T> },
                quote! {
                    pub trait AsRef<T: ?Sized> {
                        /// Converts this type into a shared reference of the (usually inferred) input type.
                        #[stable(feature = "rust1", since = "1.0.0")]
                        fn as_ref(&self) -> &T;
                    }
                },
                quote! {},
                // derive_delegate
                $derive_delegate_args,
                $derive_delegate_input,
                $derive_delegate_expected,
            }
        };
    }

    test_register_derive_delegate! {
        test_enum,
        // register
        quote! { Hello },
        quote! {
            pub trait Hello {
                fn hello(&self) -> String;
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
                fn hello(&self) -> String {
                    match self {
                        Self::A(x) => Hello::hello(x),
                        Self::B(x) => Hello::hello(x),
                    }
                }
            }
        },
    }

    test_register_derive_delegate! {
        test_enum_ref_mut_receiver,
        // register
        quote! { Hello },
        quote! {
            pub trait Hello {
                fn hello(&mut self) -> String;
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
                fn hello(&mut self) -> String {
                    match self {
                        Self::A(x) => Hello::hello(x),
                        Self::B(x) => Hello::hello(x),
                    }
                }
            }
        },
    }

    test_register_derive_delegate! {
        test_enum_consume_receiver,
        // register
        quote! { Hello },
        quote! {
            pub trait Hello {
                fn hello(self) -> String;
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
                fn hello(self) -> String {
                    match self {
                        Self::A(x) => Hello::hello(x),
                        Self::B(x) => Hello::hello(x),
                    }
                }
            }
        },
    }

    test_register_derive_delegate! {
        test_struct_with_named_field,
        // register
        quote! { Hello },
        quote! {
            pub trait Hello {
                fn hello(&self) -> String;
            }
        },
        quote! {},
        // derive_delegate
        quote! { Hello },
        quote! {
            struct Hoge {
                s: String,
            }
        },
        quote! {
            struct Hoge {
                s: String,
            }

            impl Hello for Hoge {
                fn hello(&self) -> String {
                    Hello::hello(&self.s)
                }
            }
        },
    }

    test_register_derive_delegate! {
        test_struct_with_unnamed_field,
        // register
        quote! { Hello },
        quote! {
            pub trait Hello {
                fn hello(&self) -> String;
            }
        },
        quote! {},
        // derive_delegate
        quote! { Hello },
        quote! {
            struct Hoge(String);
        },
        quote! {
            struct Hoge(String);

            impl Hello for Hoge {
                fn hello(&self) -> String {
                    Hello::hello(&self.0)
                }
            }
        },
    }

    test_register_derive_delegate! {
        test_struct_ref_mut_receiver,
        // register
        quote! { Hello },
        quote! {
            pub trait Hello {
                fn hello(&mut self) -> String;
            }
        },
        quote! {},
        // derive_delegate
        quote! { Hello },
        quote! {
            struct Hoge {
                s: String,
            }
        },
        quote! {
            struct Hoge {
                s: String,
            }

            impl Hello for Hoge {
                fn hello(&mut self) -> String {
                    Hello::hello(&mut self.s)
                }
            }
        },
    }

    test_register_derive_delegate! {
        test_struct_consume_receiver,
        // register
        quote! { Hello },
        quote! {
            pub trait Hello {
                fn hello(self) -> String;
            }
        },
        quote! {},
        // derive_delegate
        quote! { Hello },
        quote! {
            struct Hoge {
                s: String,
            }
        },
        quote! {
            struct Hoge {
                s: String,
            }

            impl Hello for Hoge {
                fn hello(self) -> String {
                    Hello::hello(self.s)
                }
            }
        },
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
        },
    }

    test_register_derive_delegate! {
        test_super_crate,
        // register
        quote! { Hello },
        quote! {
            pub trait Hello: ToString {
                fn hello(&self) -> String;
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
                fn hello(&self) -> String {
                    match self {
                        Self::A(x) => Hello::hello(x),
                        Self::B(x) => Hello::hello(x),
                    }
                }
            }
        },
    }

    test_register_register_derive_delegate! {
        test_multiple_derive,
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
        // register
        quote! { Hello },
        quote! {
            pub trait Hello: ToString {
                fn hello(&self) -> String;
            }
        },
        quote! {},
        // derive_delegate
        quote! { ToString, Hello },
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
                fn to_string(&self) -> String {
                    match self {
                        Self::A(x) => ToString::to_string(x),
                        Self::B(x) => ToString::to_string(x),
                    }
                }
            }

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

    test_as_ref! {
        test_generics_enum,
        // derive_delegate
        quote! { AsRef<str> },
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

    test_as_ref! {
        test_generics_struct,
        // derive_delegate
        quote! { AsRef<str> },
        quote! {
            struct Hoge {
                s: String,
            }
        },
        quote! {
            struct Hoge {
                s: String,
            }

            impl AsRef<str> for Hoge {
                fn as_ref(&self) -> &str {
                    AsRef::as_ref(&self.s)
                }
            }
        },
    }

    test_as_ref! {
        test_generics_specilize_complex,
        // derive_delegate
        quote! { AsRef<(dyn Fn(usize) -> usize + 'static)> },
        quote! {
            struct Hoge(Box<dyn Fn(usize) -> usize>);
        },
        quote! {
            struct Hoge(Box<dyn Fn(usize) -> usize>);

            impl AsRef<(dyn Fn(usize) -> usize + 'static)> for Hoge {
                fn as_ref(&self) -> &(dyn Fn(usize) -> usize + 'static) {
                    AsRef::as_ref(&self.0)
                }
            }
        },
    }

    test_register_derive_delegate! {
        test_generics_specilize_lifetime,
        // register
        quote! { Hello<'a, T> },
        quote! {
            pub trait Hello<'a, T> {
                fn hello(&self) -> &'a T;
            }
        },
        quote! {},
        // derive_delegate
        quote! { Hello<'p, str> },
        quote! {
            struct Hoge<'p>(&'p str);
        },
        quote! {
            struct Hoge<'p>(&'p str);

            impl Hello<'p, str> for Hoge<'p> {
                fn hello(&self) -> &'p str {
                    Hello::hello(&self.0)
                }
            }
        },
    }
}
