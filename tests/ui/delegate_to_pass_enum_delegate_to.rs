#[thin_delegate::external_trait_def]
mod __external_trait_def {
    #[thin_delegate::register]
    pub trait AsRef<T: ?Sized> {
        /// Converts this type into a shared reference of the (usually inferred) input type.
        #[stable(feature = "rust1", since = "1.0.0")]
        fn as_ref(&self) -> &T;
    }
}

#[thin_delegate::register]
enum Hoge {
    A(String),
    #[delegate_to(x => &x.0)]
    B((String, u8)),
    #[delegate_to(x => &x.1)]
    C((u8, String)),
}

#[thin_delegate::derive_delegate(external_trait_def = __external_trait_def)]
impl AsRef<str> for Hoge {}

fn main() {
    let hoge = Hoge::A("A".to_string());
    assert_eq!(hoge.as_ref(), "A");
    let hoge = Hoge::B(("B".to_string(), 0));
    assert_eq!(hoge.as_ref(), "B");
    let hoge = Hoge::C((0, "C".to_string()));
    assert_eq!(hoge.as_ref(), "C");
}