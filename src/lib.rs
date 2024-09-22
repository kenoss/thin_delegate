mod attr_remover;
mod decl_macro;
mod delegate_to_arg;
#[cfg(not(feature = "unstable_delegate_to"))]
mod delegate_to_checker;
mod delegate_to_remover;
mod derive_delegate_args;
mod gen;
mod generic_param_replacer;
mod ident_replacer;
mod punctuated_parser;
mod self_replacer;

use crate::derive_delegate_args::DeriveDelegateArgs;
use crate::gen::TraitData;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::ops::Deref;
use syn::parse_quote;
use syn::spanned::Spanned;

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

            decl_macro::define_macro_feed_trait_def_of(
                &trait_.ident,
                trait_.ident.span(),
                is_external,
                trait_,
            )
        }
        syn::Item::Struct(structenum) => decl_macro::define_macro_feed_structenum_def_of(
            &structenum.ident,
            structenum.ident.span(),
            structenum.to_token_stream(),
        ),
        syn::Item::Enum(structenum) => decl_macro::define_macro_feed_structenum_def_of(
            &structenum.ident,
            structenum.ident.span(),
            structenum.to_token_stream(),
        ),
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

    Ok(decl_macro::exec_internal_derive_delegate(
        trait_ident,
        structenum_ident,
        &external_trait_def,
        &impl_,
    ))
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
    // We'll use panic here as it is only used by this crate.

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

    gen::gen_impl(trait_, trait_path, structenum)
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
