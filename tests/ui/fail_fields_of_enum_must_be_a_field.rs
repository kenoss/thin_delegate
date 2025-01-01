#[thin_delegate::external_trait_def]
mod __external_trait_def {
    #[thin_delegate::register]
    pub trait AsRef<T: ?Sized> {
        /// Converts this type into a shared reference of the (usually inferred) input type.
        fn as_ref(&self) -> &T;
    }
}

#[thin_delegate::register]
enum Named {
    Named {
        x: String,
        y: String,
    },
}

#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def)]
impl AsRef<str> for Named {}

#[thin_delegate::register]
enum Unnamed {
    Unnamed(String, String),
}

#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def)]
impl AsRef<str> for Unnamed {}

#[thin_delegate::register]
enum Unit {
    Unit,
}

#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def)]
impl AsRef<str> for Unit {}

fn main() {}
