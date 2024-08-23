use crate::{StorableTraitData, TraitData};
use quote::ToTokens;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

#[derive(Debug, PartialEq, Eq, Hash)]
struct Key(String);

impl Key {
    fn new(path: &syn::Path) -> Self {
        let mut path = path.clone();
        path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;

        Self(path.to_token_stream().to_string())
    }
}

impl From<&syn::Path> for Key {
    fn from(path: &syn::Path) -> Self {
        Key::new(path)
    }
}

/// Store information that in necessary to pass from `register()` to `derive_delegate()`.
pub(crate) struct Storage {
    path_to_trait_data: Arc<Mutex<HashMap<Key, StorableTraitData>>>,
}

impl core::fmt::Debug for Storage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let map = self.path_to_trait_data.lock().unwrap();
        map.fmt(f)
    }
}

#[allow(clippy::type_complexity)]
static STORAGE: LazyLock<Arc<Mutex<HashMap<Key, StorableTraitData>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

impl Storage {
    /// Acquire an accessor to compile-time global storage.
    pub fn global() -> Self {
        Self {
            path_to_trait_data: LazyLock::force(&STORAGE).clone(),
        }
    }

    pub fn store(&mut self, path: &syn::Path, trait_data: &TraitData) -> syn::Result<()> {
        let key = path.into();
        let mut map = self.path_to_trait_data.lock().unwrap();

        if map.contains_key(&key) {
            return Err(syn::Error::new_spanned(
                path,
                format!(
                    "type name conflicted, arleady registered: path = {path}",
                    path = path.to_token_stream(),
                ),
            ));
        }

        map.insert(key, trait_data.into());

        Ok(())
    }

    pub fn get(&mut self, path: &syn::Path) -> Option<TraitData> {
        let key = path.into();
        let map = self.path_to_trait_data.lock().unwrap();
        let trait_data = map.get(&key)?.into();
        Some(trait_data)
    }
}

#[cfg(test)]
mod test_storage {
    use super::*;

    pub(crate) struct TestStorageFactory {
        path_to_trait_data: Arc<Mutex<HashMap<Key, StorableTraitData>>>,
    }

    impl TestStorageFactory {
        pub fn new() -> Self {
            Self {
                path_to_trait_data: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn factory(&mut self) -> Storage {
            Storage {
                path_to_trait_data: self.path_to_trait_data.clone(),
            }
        }
    }
}

#[cfg(test)]
pub(crate) use test_storage::TestStorageFactory;
