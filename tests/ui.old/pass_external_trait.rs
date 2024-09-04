// TODO: Introduce safe mechanism not to leak `STORAGE` for `register()` for external traits.

mod private_for_thin_delegate {
    #[thin_delegate::register(ToString)]
    pub trait ToString {
        /// Converts the given value to a `String`.
        ///
        /// # Examples
        ///
        /// ```
        /// let i = 5;
        /// let five = String::from("5");
        ///
        /// assert_eq!(five, i.to_string());
        /// ```
        #[rustc_conversion_suggestion]
        #[stable(feature = "rust1", since = "1.0.0")]
        #[cfg_attr(not(test), rustc_diagnostic_item = "to_string_method")]
        fn to_string(&self) -> String;
    }
}

#[thin_delegate::derive_delegate(ToString)]
enum Hoge {
    A(String),
    B(char),
}

fn main() {
    let hoge = Hoge::A("a".to_string());
    assert_eq!(hoge.to_string(), "a");

    let hoge = Hoge::B('b');
    assert_eq!(hoge.to_string(), "b");
}
