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
    B((String, u8)),
    #[delegate_to(x => &x.1)]
    C((u8, String)),
}

fn main() {
    let hoge = Hoge::A("A".to_string());
    assert_eq!(hoge.as_ref(), "A");
    let hoge = Hoge::B(("B".to_string(), 0));
    assert_eq!(hoge.as_ref(), "B");
    let hoge = Hoge::C((0, "C".to_string()));
    assert_eq!(hoge.as_ref(), "C");
}
