// TODO: Make `register()` is usable for trait definition.
mod private_for_thin_delegate {
    #[thin_delegate::register(AsRef)]
    pub trait AsRef<T: ?Sized> {
        /// Converts this type into a shared reference of the (usually inferred) input type.
        #[stable(feature = "rust1", since = "1.0.0")]
        fn as_ref(&self) -> &T;
    }
}

#[thin_delegate::derive_delegate(AsRef<str>)]
enum Hoge {
    A(String),
    #[delegate_to(x => &x.0)]
    #[delegate_to(x => &x.0)]
    B((String, u8)),
}

fn main() {}
