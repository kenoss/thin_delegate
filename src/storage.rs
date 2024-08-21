use crate::{FnIngredient, StorableFnIngredient};
use quote::ToTokens;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

#[derive(PartialEq, Eq, Hash)]
struct PathAsString(String);

static STORAGE: LazyLock<Mutex<HashMap<PathAsString, Vec<StorableFnIngredient>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) fn store(path: &syn::Path, fn_ingredients: &[FnIngredient]) -> syn::Result<()> {
    let key = PathAsString(path.to_token_stream().to_string());

    if STORAGE.lock().unwrap().contains_key(&key) {
        return Err(syn::Error::new_spanned(
            path,
            format!(
                "type name conflicted, arleady registered: path = {path}",
                path = path.to_token_stream(),
            ),
        ));
    }

    let value = fn_ingredients.iter().map(|x| x.into()).collect();

    STORAGE.lock().unwrap().insert(key, value);

    Ok(())
}

pub(crate) fn get(path: &syn::Path) -> Option<Vec<FnIngredient>> {
    let key = PathAsString(path.to_token_stream().to_string());
    let map = STORAGE.lock().unwrap();
    let fn_ingredients = map.get(&key)?.iter().map(|x| x.into()).collect();
    Some(fn_ingredients)
}
