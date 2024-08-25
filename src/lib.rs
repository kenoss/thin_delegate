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
pub(crate) struct TraitData {
    trait_path: syn::Path,
    generics: syn::Generics,
    sigs: Vec<syn::Signature>,
}

struct FnIngredient<'a> {
    trait_path: &'a syn::Path,
    sig: &'a syn::Signature,
}

#[derive(Debug)]
pub(crate) struct StorableTraitData {
    trait_path: String,
    generics: String,
    sigs: Vec<String>,
}

impl From<&TraitData> for StorableTraitData {
    fn from(x: &TraitData) -> Self {
        Self {
            trait_path: x.trait_path.to_token_stream().to_string(),
            generics: x.generics.to_token_stream().to_string(),
            sigs: x
                .sigs
                .iter()
                .map(|sig| sig.to_token_stream().to_string())
                .collect(),
        }
    }
}

impl From<&StorableTraitData> for TraitData {
    fn from(x: &StorableTraitData) -> Self {
        Self {
            trait_path: syn::parse2::<syn::Path>(x.trait_path.parse().unwrap()).unwrap(),
            generics: syn::parse2::<syn::Generics>(x.generics.parse().unwrap()).unwrap(),
            sigs: x
                .sigs
                .iter()
                .map(|sig| syn::parse2::<syn::Signature>(sig.parse().unwrap()).unwrap())
                .collect(),
        }
    }
}

impl TraitData {
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
            "arguments must not be empty: `#[thin_delegate::register(<path>, ...)]`",
        ));
    }

    let path = syn::parse2::<syn::Path>(args.clone()).map_err(|_| {
        syn::Error::new_spanned(
            args,
            "type argument expected: `#[thin_delegate::register(<path>, ...)]`",
        )
    })?;

    let syn::Item::Trait(trait_) = item else {
        return Err(syn::Error::new(item.span(), "expected `trait ...`"));
    };

    if path.segments.last().unwrap().arguments != syn::PathArguments::None {
        return Err(syn::Error::new_spanned(
            path,
            "argument must be a path without generic paramteres, like in `use ...`",
        ));
    }

    let sigs = trait_
        .items
        .iter()
        .filter_map(|x| {
            let syn::TraitItem::Fn(fn_) = x else {
                return None;
            };

            Some(fn_.sig.clone())
        })
        .collect_vec();
    let trait_data = TraitData {
        trait_path: path.clone(),
        generics: trait_.generics.clone(),
        sigs,
    };
    trait_data.validate()?;

    storage.store(&path, &trait_data)?;

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
            "arguments must not be empty `#[thin_delegate::derive_delegate(<path>, ...)]`",
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
    let Some(trait_data) = storage.get(path) else {
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

    if trait_data.sigs.is_empty() {
        return Ok(quote! {});
    }

    let generic_param_replacer = GenericParamReplacer::new(
        &trait_data.generics,
        &path.segments.last().unwrap().arguments,
    )?;

    let funcs = trait_data
        .fn_ingredients()
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

                    Ok(quote! {
                        Self::#variant_ident { #ident } => #trait_path::#method_ident(#ident #(,#args)*)
                    })
                }
                syn::Fields::Unnamed(fields) => {
                    if fields.unnamed.len() != 1 {
                        return Err(syn::Error::new_spanned(
                            &variant.fields,
                            "fields of enum variant must be a field",
                        ));
                    }

                    Ok(quote! {
                        Self::#variant_ident(x) => #trait_path::#method_ident(x #(,#args)*)
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
                quote! { AsRef },
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
        r#enum,
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
                Named {
                    named: String,
                },
                Unnamed(char),
            }
        },
        quote! {
            enum Hoge {
                Named {
                    named: String,
                },
                Unnamed(char),
            }

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

    test_register_derive_delegate! {
        enum_ref_mut_receiver,
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
        enum_consume_receiver,
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
        struct_with_named_field,
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
        struct_with_unnamed_field,
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
        struct_ref_mut_receiver,
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
        struct_consume_receiver,
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
        method_with_args,
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
        super_crate,
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
        multiple_derive,
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
        generics_enum,
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
        generics_struct,
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
        generics_specilize_complex,
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
        generics_specilize_lifetime,
        // register
        quote! { Hello },
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
