#[thin_delegate::external_trait_def]
mod __external_trait_def {
    #[thin_delegate::register]
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

#[thin_delegate::register]
enum Hoge {
    A(String),
    B(char),
}

#[thin_delegate::derive_delegate(external_trait_def = __external_trait_def)]
impl ToString for Hoge {}

fn main() {
    let hoge = Hoge::A("a".to_string());
    assert_eq!(hoge.to_string(), "a");

    let hoge = Hoge::B('b');
    assert_eq!(hoge.to_string(), "b");
}
