//! Auto implementation of trait functions by delegation to inner types
//!
//! This crate provides attribute macros that supports to define trait functions by delegation to
//! inner types.
//!
//! - `#[thin_delegate::register]`: Registers definitions of trait, struct and enum.
//! - `#[thin_delegate::fill_delegate]`: Derives and fills `impl Trait for StructEnum` by delegation.
//! - `#[thin_delegate::external_trait_def]`: Imports trait definitions in external crates.
//!
//! There exist similar crates. See [comparison](#comparison) for more details.
//!
//! See also related RFCs:
//! [rfcs#1406](https://github.com/rust-lang/rfcs/pull/1406),
//! [rfcs#2393](https://github.com/rust-lang/rfcs/pull/2393),
//! [rfcs#3530](https://github.com/rust-lang/rfcs/pull/3530).
//!
//! ## Example
//!
//! ```
//! #[thin_delegate::register]
//! trait AnimalI {
//!     fn sound(&self) -> String;
//!     fn walk(&mut self, pos: usize) -> usize;
//! }
//!
//! #[thin_delegate::register]
//! struct Duck(String);
//!
//! #[thin_delegate::register]
//! struct Cat {
//!     sound: String,
//!     speed: usize,
//! }
//!
//! #[thin_delegate::register]
//! enum Animal {
//!     Duck(Duck),
//!     Cat(Cat),
//! }
//!
//! // Implement delegatee manually.
//! impl AnimalI for String {
//!     fn sound(&self) -> String {
//!         self.clone()
//!     }
//!
//!     // String doesn't walk.
//!     fn walk(&mut self, _pos: usize) -> usize {
//!         unimplemented!();
//!     }
//! }
//!
//! // Delegate all methods to `String`. Leave `walk()` umimplemented.
//! // Delegation of a struct with single field is automatic.
//! #[thin_delegate::fill_delegate]
//! impl AnimalI for Duck {}
//!
//! // Delegate `sound()` to `sound: String`. Implement `walk()` manually.
//! // Delegation of a struct with multiple fields is ambiguous. Needs to designate `scheme`.
//! #[thin_delegate::fill_delegate(scheme = |f| f(&self.sound))]
//! impl AnimalI for Cat {
//!     fn walk(&mut self, pos: usize) -> usize {
//!         pos + self.speed
//!     }
//! }
//!
//! // Delegate all methods to each arms `Duck` and `Cat`.
//! // Delegation of an enum is automatic.
//! #[thin_delegate::fill_delegate]
//! impl AnimalI for Animal {}
//!
//! let duck = Duck("quack".to_string());
//! let mut cat = Cat { sound: "mew".to_string(), speed: 1 };
//! let mut neko = Cat { sound: "nya-nya-".to_string(), speed: 2 };
//! assert_eq!(duck.sound(), "quack");
//! assert_eq!(cat.sound(), "mew");
//! assert_eq!(cat.walk(10), 11);
//! assert_eq!(neko.sound(), "nya-nya-");
//! assert_eq!(neko.walk(10), 12);
//! let duck = Animal::Duck(duck);
//! let mut cat = Animal::Cat(cat);
//! let mut neko = Animal::Cat(neko);
//! assert_eq!(duck.sound(), "quack");
//! assert_eq!(cat.sound(), "mew");
//! assert_eq!(cat.walk(10), 11);
//! assert_eq!(neko.sound(), "nya-nya-");
//! assert_eq!(neko.walk(10), 12);
//! ```
//!
//! See [tests](https://github.com/kenoss/thin_delegate/blob/main/tests/ui) for more examples and
//! [sabiniwm](https://github.com/kenoss/sabiniwm) for real world examples.
//!
//! - `external_trait_def`
//!   - [Import external trait definition](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_external_trait_def.rs)
//!   - [Import external trait definition with `use`s](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_external_trait_def_with_uses.rs)
//! - `scheme`
//!   - [struct](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_scheme.rs)
//!   - [enum](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_scheme_enum.rs)
//! - Trait admits
//!   - Generics
//!     - [complex type parameter](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_generics_specialize_complex.rs)
//!     - [const](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_generics_const.rs)
//!   - Trait bounds
//!     - [super trait](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_super_trait.rs)
//!     - [`where` and complex method argument](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_prevent_ambiguous_generic_params.rs)
//! - [Only fills not implemented methods](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_items_in_impl.rs)
//!
//! ## How it works
//!
//! 1. `#[thin_delegate::register]` defines a declarative macro for each trait/struct/enum definition.
//! 2. `#[thin_delegate::fill_delegate]` collects related definitions by using those declarative macros and CPS,
//!    and then calls an attribute macro `#[thin_delegate::__internal__fill_delegate]`.
//! 3. `#[thin_delegate::__internal__fill_delegate]` fills `impl Trait for StructEnum {...}`.
//!
//! See [src/decl_macro.rs](https://github.com/kenoss/thin_delegate/blob/main/src/decl_macro.rs) for more details.
//!
//! ## FAQ
//!
//! ### What is an error like <code>error: cannot find macro \`__thin_delegate__feed_trait_def_of_Hello\` in this scope</code>?
//!
//! In the above step 2, `#[thin_delegate::fill_delegate]` needs some declarative macros.
//! This error reports that rustc couldn't find the macro.
//!
//! Recommended actions:
//!
//! - Make sure that your trait/struct/enum is qualified with `#[thin_delegate::register]` correctly.
//! - If you are using an external trait definition, make sure that a path of a module is given by
//!   an argument `external_trait_def` of `#[thin_delegate::fill_delegate]` and the module is
//!   qualified with `#[thin_delegate::external_trait_def]`.
//!
//! See `fail_register_for_*.rs` in [tests](https://github.com/kenoss/thin_delegate/tree/main/tests/ui)
//! for the exact error messages.
//!
//! ## Performance
//!
//! Note that using `enum` is more performant than `Box<dyn Trait>` in general case.
//! (The main reason is not using vtable. One can expect branch prediction works for `match` in most-inner loops.)
//! See also
//! [benchmark of `enum_dispatch`](https://docs.rs/enum_dispatch/0.3.13/enum_dispatch/index.html#performance).
//! It would be an option if you need just a polymorphism closed in your application. See
//! [`Backend` in sabiniwm](https://github.com/kenoss/sabiniwm/blob/main/crates/sabiniwm/src/backend/mod.rs)
//! for example (while the main reason for using `enum` is not performance in this case).
//!
//! ## Comparison
//!
//! ### [enum_dispatch](https://crates.io/crates/enum_dispatch)
//!
//! - [Limitations](https://github.com/kenoss/kenoss.github.io/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch)
//!   - Doesn't support, e.g. external traits and [generics](https://github.com/kenoss/kenoss.github.io/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/fail_generics.rs).
//! - Implementation uses not safe mechanism (Using global variable in proc macro)
//!
//! See also [documentation of `enum_delegate`](https://docs.rs/enum_delegate/0.2.0/enum_delegate/#comparison-with-enum_dispatch).
//!
//! ### [enum_delegate](https://crates.io/crates/enum_delegate) (< v0.3.0)
//!
//! - [Limitations](https://github.com/kenoss/kenoss.github.io/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v020)
//!   - Doesn't support, e.g. [super traits](https://github.com/kenoss/kenoss.github.io/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v020/fail_super_trait.rs).
//!
//! See also [limitations](https://docs.rs/enum_delegate/0.2.0/enum_delegate/#limitations).
//!
//! ### [enum_delegate](https://crates.io/crates/enum_delegate) (>= v0.3.0)
//!
//! - [Limitations](https://github.com/kenoss/kenoss.github.io/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v030)
//!   - Doesn't support, e.g.
//!     [super traits](https://github.com/kenoss/kenoss.github.io/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v030/fail_not_supported_super_trait.rs),
//!     [associated const/type](https://github.com/kenoss/kenoss.github.io/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v030/fail_not_supported_associated_const.rs).
//! - Implementation uses very restricted mechanism.
//!
//! See also [limitations](https://gitlab.com/dawn_app/enum_delegate/tree/f5bcaf45#limitations).
//!
//! ### [auto-delegate](https://crates.io/crates/auto-delegate)
//!
//! - [Limitations](https://github.com/kenoss/kenoss.github.io/content/blog/2025-01-01-thin_delegate_aux/tests/ui/auto-delegate)
//!   - Doesn't support, e.g. super traits,
//!     [associated const/type](https://github.com/kenoss/kenoss.github.io/content/blog/2025-01-01-thin_delegate_aux/tests/ui/auto-delegate/fail_not_supported_associated_const.rs),
//!     [delegating to `std` types](https://github.com/kenoss/kenoss.github.io/content/blog/2025-01-01-thin_delegate_aux/tests/ui/auto-delegate/fail_conflict_impl_trait_for_field.rs).
//! - Supports methods without a receiver.
//! - Implementation uses very restricted mechanism.
//!
//! ### [ambassader](https://crates.io/crates/ambassador)
//!
//! - Competitive. I recommend it if you doesn't need features/APIs of `thin_delegate`.
//!
//! ### [portrait](https://crates.io/crates/portrait)
//!
//! - Exposes a macro with the same name to the struct/enum.

mod attr_remover;
mod decl_macro;
mod external_trait_def_args;
mod fill_delegate_args;
mod fn_call_replacer;
mod gen;
mod generic_param_replacer;
mod self_replacer;

use crate::external_trait_def_args::ExternalTraitDefArgs;
use crate::fill_delegate_args::FillDelegateArgs;
use crate::gen::TraitData;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::ops::Deref;
use syn::parse_quote;
use syn::spanned::Spanned;

/// An attribute macro marking a module as "external trait definitions"
///
/// See [toplevel documentation](./) for fundamental usage.
///
/// ## Arguments
///
/// ### `with_uses = <bool>`
///
/// If `true`, `#[thin_delegate::fill_delegate]` wraps `impl Trait for StructEnum` within a module
/// and expands the imports `use ...` in the orginal module to the expanded module. It is convenient
/// to copy&paste the original definition as is.
///
/// Defaults to `false`.
///
/// See also [example](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_external_trait_def_with_uses.rs).
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
    let args = syn::parse2::<ExternalTraitDefArgs>(args)?;

    let e = syn::Error::new(item.span(), "expected `mod ... { ... }`");
    let item = syn::parse2::<syn::Item>(item).map_err(|_| e.clone())?;
    let syn::Item::Mod(mut mod_) = item else {
        return Err(e);
    };
    let Some(ref mut content) = mod_.content else {
        return Err(e);
    };

    let uses = if args.with_uses {
        let mut uses = vec![];

        for mut item in &mut content.1 {
            #[allow(clippy::single_match)]
            match &mut item {
                syn::Item::Use(use_) => {
                    use_.attrs.push(parse_quote! { #[allow(unused)] });
                    uses.push(use_.clone());
                }
                _ => {}
            }
        }

        Some(quote! { #(#uses)* })
    } else {
        None
    };

    for item in &mut content.1 {
        #[allow(clippy::single_match)]
        match item {
            syn::Item::Trait(ref mut trait_) => {
                let attr = parse_quote! {
                    #[::thin_delegate::__internal__is_external_marker]
                };
                trait_.attrs.push(attr);

                if let Some(uses) = &uses {
                    let attr = parse_quote! {
                        #[::thin_delegate::__internal__with_uses(#uses)]
                    };
                    trait_.attrs.push(attr);
                }
            }
            _ => {}
        }
    }

    Ok(quote! { #mod_ })
}

/// Do not use. This is only used from `thin_delegate` crate internal.
#[doc(hidden)]
#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn __internal__is_external_marker(
    _args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item: TokenStream = item.into();
    syn::Error::new_spanned(item, "#[thin_delegate::register] missing for trait")
        .into_compile_error()
        .into()
}

/// An attribute macro registering a definition of trait/struct/enum for `#[thin_delegate::fill_delegate]`
///
/// See [toplevel documentation](./) for fundamental usage.
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
            #[allow(non_snake_case)]
            let __internal__is_external_marker: syn::Attribute = parse_quote! {
                #[::thin_delegate::__internal__is_external_marker]
            };
            trait_
                .attrs
                .iter()
                .any(|attr| *attr == __internal__is_external_marker)
        }
        _ => false,
    };
    let macro_def = match &item {
        syn::Item::Trait(trait_) => {
            let trait_path = syn::Path::from(syn::PathSegment::from(trait_.ident.clone()));
            // Note that `args` and `trait_path` here are kinds of dummy. It's just used for validation.
            let trait_data = TraitData::new(&FillDelegateArgs::default(), trait_, trait_path);
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
            &item,
        ),
        syn::Item::Enum(structenum) => decl_macro::define_macro_feed_structenum_def_of(
            &structenum.ident,
            structenum.ident.span(),
            &item,
        ),
        _ => {
            return Err(syn::Error::new_spanned(
                item,
                "expected `trait ...` or `struct ...` or `enum ...`",
            ));
        }
    };

    attr_remover::relplace_attr_with_do_nothing_in_item(
        parse_quote! { ::thin_delegate::__internal__is_external_marker },
        &mut item,
    );

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

/// An attribute macro deriving `impl` by delegation to an inner field
///
/// See [toplevel documentation](./) for fundamental usage.
///
/// ## Arguments
///
/// ### `delegate_fn_with_default_impl = <bool>`
///
/// By default (`false`), it doesn't fill trait functions with default implementation.
/// If `true`, it fills them by delegation.
///
/// See also [example](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_delegate_fn_with_default_impl.rs).
///
/// ### `external_trait_def = <path>`
///
/// Designates a path of module that is qualified by `#[thin_delegate::external_trait_def]` and
/// contains the trait definition qualified with `#[thin_delegate::register]`.
///
/// How it works (See also [How it works](./index.html#how-it-works).):
/// `#[thin_delegate::register]` defines a macro that contains information of a trait.
/// Normally, `#[thin_delegate::fill_delegate]` searches the macro in current module.
/// The argument `external_trait_def = path::to::mod` modifies it to search
/// `path::to::mod::<macro>`.
///
/// ### `scheme = <closure-like>`
///
/// Defines a scheme to generate implementations of methods instead of the default generation
/// algorithm.
///
/// ```
/// #[thin_delegate::register]
/// trait Hello {
///     fn hello(&self) -> String;
/// }
///
/// impl Hello for String {
///     fn hello(&self) -> String {
///         self.clone()
///     }
/// }
///
/// #[thin_delegate::register]
/// struct Hoge(char);
///
/// impl Hoge {
///     fn key(&self) -> String {
///         format!("key-{}", self.0)
///     }
/// }
///
/// #[thin_delegate::fill_delegate(scheme = |f| f(&self.key()))]
/// impl Hello for Hoge {}
/// ```
///
/// A scheme is like a closure, but actually it is used more lexically; each occurrence of the
/// parameter `f` is replaced with a trait method `Hello::hello`, like templates in C++.
///
/// See also [example](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_scheme_enum.rs).
#[proc_macro_attribute]
pub fn fill_delegate(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item: TokenStream = item.into();

    match fill_delegate_aux(args.into(), item.clone()) {
        Ok(x) => x.into(),
        Err(e) => TokenStream::from_iter([e.into_compile_error(), item]).into(),
    }
}

fn fill_delegate_aux(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let args_as_tokenstream = args.clone();
    let args = syn::parse2::<FillDelegateArgs>(args)?;
    args.validate()?;

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

    Ok(decl_macro::exec_internal_fill_delegate(
        trait_ident,
        structenum_ident,
        &args.external_trait_def,
        args_as_tokenstream,
        &impl_,
    ))
}

/// Do not use. This is only used from `thin_delegate` crate internal.
#[doc(hidden)]
#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn __internal__fill_delegate(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match internal_fill_delegate_aux(args.into(), input.into()) {
        Ok(x) => x.into(),
        Err(e) => TokenStream::from_iter([e.into_compile_error()]).into(),
    }
}

fn internal_fill_delegate_aux(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    // We'll use panic here as it is only used by this crate.

    let args = syn::parse2::<FillDelegateArgs>(args)?;
    args.validate()?;

    let item = syn::parse2::<syn::Item>(item.clone()).unwrap();
    let syn::Item::Mod(mod_) = item else {
        panic!();
    };
    let content = mod_.content.unwrap().1;
    assert_eq!(content.len(), 3);
    let mut it = content.into_iter();
    let syn::Item::Trait(trait_) = it.next().unwrap() else {
        panic!();
    };
    let structenum = it.next().unwrap();
    let syn::Item::Impl(impl_) = it.next().unwrap() else {
        panic!();
    };
    let Some((_, trait_path, _)) = &impl_.trait_ else {
        panic!()
    };

    let with_uses_path = parse_quote! { ::thin_delegate::__internal__with_uses };
    let uses = trait_.attrs.iter().find_map(|attr| match &attr.meta {
        syn::Meta::List(meta) if meta.path == with_uses_path => Some(meta.tokens.clone()),
        _ => None,
    });

    let mod_name = format!(
        "__thin_delegate__impl_{}_for_{}",
        trait_path.to_token_stream(),
        impl_.self_ty.to_token_stream()
    );
    // For simplicity, we allow only ascii characters so far.
    //
    // TODO: Support non ascii characters.
    let mod_name = mod_name.replace(|c: char| !c.is_ascii_alphabetic(), "_");
    let mod_name = syn::Ident::new_raw(&mod_name, Span::call_site());

    let impl_ = gen::gen_impl(&args, &trait_, &trait_path.clone(), &structenum, impl_)?;

    if let Some(uses) = uses {
        Ok(quote! {
            mod #mod_name {
                use super::*;

                #uses

                #impl_
            }
        })
    } else {
        Ok(quote! {
            #impl_
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! compare_result {
        ($got:expr, $expected:expr) => {
            let got: syn::Result<TokenStream> = $got;
            let expected: syn::Result<TokenStream> = $expected;
            assert_eq!(
                got.map(|x| x.to_string()).map_err(|e| e.to_string()),
                expected.map(|x| x.to_string()).map_err(|e| e.to_string())
            );
        };
    }

    macro_rules! test_internal_fill_delegate {
        (
            $test_name:ident,
            $args:expr,
            $input:expr,
            $expected:expr,
        ) => {
            #[test]
            fn $test_name() -> syn::Result<()> {
                let args: TokenStream = $args;
                let input: TokenStream = $input;
                let expected: TokenStream = $expected;

                let input = quote! {
                    mod __thin_delegate__test_mod {
                        #input
                    }
                };
                compare_result!(internal_fill_delegate_aux(args, input), Ok(expected));

                Ok(())
            }
        };
    }

    test_internal_fill_delegate! {
        r#enum,
        quote! {},
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

    test_internal_fill_delegate! {
        enum_ref_mut_receiver,
        quote! {},
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

    test_internal_fill_delegate! {
        enum_consume_receiver,
        quote! {},
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

    test_internal_fill_delegate! {
        struct_with_named_field,
        quote! {},
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

    test_internal_fill_delegate! {
        struct_with_unnamed_field,
        quote! {},
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

    test_internal_fill_delegate! {
        struct_ref_mut_receiver,
        quote! {},
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

    test_internal_fill_delegate! {
        struct_consume_receiver,
        quote! {},
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

    test_internal_fill_delegate! {
        method_with_args,
        quote! {},
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

    test_internal_fill_delegate! {
        super_trait,
        quote! {},
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

    test_internal_fill_delegate! {
        generics_enum,
        quote! {},
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
                        Self::A(x) => AsRef::<str>::as_ref(x),
                        Self::B(x) => AsRef::<str>::as_ref(x),
                    }
                }
            }
        },
    }

    test_internal_fill_delegate! {
        generics_struct,
        quote! {},
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
                    AsRef::<str>::as_ref(&self.s)
                }
            }
        },
    }

    test_internal_fill_delegate! {
        generics_specilize_complex,
        quote! {},
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
                    AsRef::<(dyn Fn(usize) -> usize + 'static)>::as_ref(&self.0)
                }
            }
        },
    }

    test_internal_fill_delegate! {
        generics_specilize_lifetime,
        quote! {},
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
                    Hello::<'p, str>::hello(&self.0)
                }
            }
        },
    }

    test_internal_fill_delegate! {
        items_in_impl,
        quote! {},
        quote! {
            trait Hello {
                type Return;

                const HAS_DEFAULT: &'static str = "HAS_DEFAULT";
                const NEED_TO_FILL: &'static str;

                // `thin_delegate` only can fill associated functions.
                fn filled(&self) -> Self::Return;
                fn override_(&self) -> Self::Return;
                fn skipped_if_fn_has_default_impl(&self) -> Self::Return {
                    self.filled()
                }
            }

            struct Hoge(String);

            impl Hello for Hoge {
                // It can handle associated types in impl.
                //
                // You need to specify them by yourself as if you don't use `thin_delegate`.
                type Return = String;

                // It can handle associated consts in impl.
                //
                // You need to specify them by yourself as if you don't use `thin_delegate`.
                const NEED_TO_FILL: &'static str = "Hoge";

                // It can handle associated functions in impl.
                //
                // If an impl doesn't has an associated function (`filled()`), it is filled.
                // If an impl has an associated function (`override_()`), it is used.
                fn override_(&self) -> Self::Return {
                    self.0.override_().to_uppercase()
                }

                // It doesn't fill `skipped_if_fn_has_default_impl()` as it has default implementation.
            }
        },
        quote! {
            impl Hello for Hoge {
                type Return = String;

                const NEED_TO_FILL: &'static str = "Hoge";

                fn override_(&self) -> Self::Return {
                    self.0.override_().to_uppercase()
                }

                fn filled(&self) -> Self::Return {
                    Hello::filled(&self.0)
                }
            }
        },
    }

    test_internal_fill_delegate! {
        delegate_fn_with_default_impl,
        quote! {delegate_fn_with_default_impl = true},
        quote! {
            trait Hello {
                fn skipped_if_fn_has_default_impl(&self) -> Self::Return {
                    self.filled()
                }
            }

            struct Hoge(String);

            impl Hello for Hoge {
                // It fills `skipped_if_fn_has_default_impl()` if `delegate_fn_with_default_impl = true`.
            }
        },
        quote! {
            impl Hello for Hoge {
                fn skipped_if_fn_has_default_impl(&self) -> Self::Return {
                    Hello::skipped_if_fn_has_default_impl(&self.0)
                }
            }
        },
    }

    test_internal_fill_delegate! {
        macro_in_impl,
        quote! {},
        quote! {
            trait Hello {
                fn filled(&self) -> String;
                fn override_(&self) -> String;
            }

            struct Hoge(String);

            impl Hello for Hoge {
                // `thin_delegate` can't recognize associated functions generated by macros because
                // the expansion of `#[thin_delegate::fill_delegate]` is earlier than ones of
                // macros inside.
                gen_override! {self, {
                    self.0.override_().to_uppercase()
                }}
            }
        },
        quote! {
            impl Hello for Hoge {
                gen_override! {self, {
                    self.0.override_().to_uppercase()
                }}

                fn filled(&self) -> String {
                    Hello::filled(&self.0)
                }

                fn override_(&self) -> String {
                    Hello::override_(&self.0)
                }
            }
        },
    }

    test_internal_fill_delegate! {
        scheme,
        quote! {
            scheme = |f| f(&self.key())
        },
        quote! {
            pub trait Hello {
                fn hello(&self, prefix: &str) -> String;
            }

            struct Hoge(String);

            impl Hello for Hoge {}
        },
        quote! {
            impl Hello for Hoge {
                fn hello(&self, prefix: &str) -> String {
                    Hello::hello(&self.key(), prefix)
                }
            }
        },
    }

    test_internal_fill_delegate! {
        scheme_enum,
        quote! {
            scheme = |f| {
                match self {
                    Self::A(s) => f(&format!("{s}{s}")),
                    Self::B(c) => f(c),
                }
            }
        },
        quote! {
            pub trait Hello {
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
                    {
                        match self {
                            Self::A(s) => Hello::hello(&format!("{s}{s}"), prefix),
                            Self::B(c) => Hello::hello(c, prefix),
                        }
                    }
                }
            }
        },
    }
}
