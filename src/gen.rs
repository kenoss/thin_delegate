use crate::delegate_to_arg::DelegateToArg;
use crate::derive_delegate_args::DeriveDelegateArgs;
use crate::generic_param_replacer::GenericParamReplacer;
use crate::{fn_call_replacer, ident_replacer, self_replacer};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::parse_quote;
use syn::spanned::Spanned;

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

    pub fn validate(&self) -> syn::Result<()> {
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

pub(crate) fn gen_impl(
    args: &DeriveDelegateArgs,
    trait_: &syn::ItemTrait,
    trait_path: &syn::Path,
    structenum: &syn::Item,
    impl_: syn::ItemImpl,
) -> syn::Result<TokenStream> {
    let trait_data = TraitData::new(trait_, trait_path.clone());

    let generic_param_replacer = GenericParamReplacer::new(
        &trait_data.generics,
        &trait_path.segments.last().unwrap().arguments,
    )?;

    let mut func_idents = HashSet::new();
    for item in &impl_.items {
        let syn::ImplItem::Fn(func) = item else {
            continue;
        };
        func_idents.insert(func.sig.ident.clone());
    }

    let mut funcs = vec![];
    for fn_ingredient in trait_data.fn_ingredients() {
        if func_idents.contains(&fn_ingredient.sig.ident) {
            continue;
        }
        funcs.push(gen_impl_fn(
            args,
            &generic_param_replacer,
            structenum,
            fn_ingredient,
        )?);
    }

    let mut impl_ = impl_;
    impl_.items.append(&mut funcs);

    Ok(quote! { #impl_ })
}

fn gen_impl_fn(
    args: &DeriveDelegateArgs,
    generic_param_replacer: &GenericParamReplacer,
    item: &syn::Item,
    fn_ingredient: FnIngredient<'_>,
) -> syn::Result<syn::ImplItem> {
    if let Some(impl_) = gen_impl_fn_scheme(args, generic_param_replacer, &fn_ingredient) {
        return Ok(impl_);
    }

    match item {
        syn::Item::Enum(enum_) => gen_impl_fn_enum(generic_param_replacer, enum_, &fn_ingredient),
        syn::Item::Struct(struct_) => {
            gen_impl_fn_struct(generic_param_replacer, struct_, &fn_ingredient)
        }
        _ => Err(syn::Error::new(
            item.span(),
            "expected `enum ...` or `struct ...`",
        )),
    }
}

fn gen_impl_fn_scheme(
    args: &DeriveDelegateArgs,
    generic_param_replacer: &GenericParamReplacer,
    fn_ingredient: &FnIngredient<'_>,
) -> Option<syn::ImplItem> {
    let (arg, body) = args.scheme_arg_and_body()?;

    let trait_path = &fn_ingredient.trait_path;
    let method_ident = &fn_ingredient.sig.ident;
    let path = parse_quote! { #trait_path::#method_ident };

    let non_receiver_args = fn_ingredient
        .args()
        .iter()
        .map(|x| {
            let path = syn::Path::from(syn::PathSegment::from(x.ident.clone()));
            syn::Expr::from(syn::ExprPath {
                attrs: vec![],
                qself: None,
                path,
            })
        })
        .collect();
    let body = fn_call_replacer::replace_fn_call_in_expr(
        arg.clone(),
        path,
        non_receiver_args,
        body.clone(),
    );

    let sig = generic_param_replacer.replace_signature(fn_ingredient.sig.clone());
    let sig = self_replacer::make_self_hygienic_in_signature(sig);
    Some(parse_quote! {
        #sig {
            #body
        }
    })
}

fn gen_impl_fn_enum(
    generic_param_replacer: &GenericParamReplacer,
    enum_: &syn::ItemEnum,
    fn_ingredient: &FnIngredient<'_>,
) -> syn::Result<syn::ImplItem> {
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
    Ok(parse_quote! {
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
    fn_ingredient: &FnIngredient<'_>,
) -> syn::Result<syn::ImplItem> {
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
    Ok(parse_quote! {
        #sig {
            #trait_path::#method_ident(#receiver #(,#args)*)
        }
    })
}
