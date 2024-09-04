// TODO: Make `register()` is usable for trait definition.
mod private_for_thin_delegate {
    #[thin_delegate::register(AsRef)]
    pub trait AsRef<T: ?Sized> {
        /// Converts this type into a shared reference of the (usually inferred) input type.
        fn as_ref(&self) -> &T;
    }
}

#[thin_delegate::derive_delegate(AsRef<str>)]
enum Named {
    Named {
        x: String,
        y: String,
    },
}

#[thin_delegate::derive_delegate(AsRef<str>)]
enum Unnamed {
    Unnamed(String, String),
}

#[thin_delegate::derive_delegate(AsRef<str>)]
enum Unit {
    Unit,
}

fn main() {}
