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
pub trait Hello: ToString {
    fn hello(&self) -> String;
}

impl Hello for String {
    fn hello(&self) -> String {
        format!("hello, {}", &self.to_string())
    }
}

impl Hello for char {
    fn hello(&self) -> String {
        format!("hello, {}", &self.to_string())
    }
}

#[thin_delegate::register]
enum Hoge {
    A(String),
    B(char),
}

#[thin_delegate::derive_delegate(external_trait_def = __external_trait_def)]
impl ToString for Hoge {}

#[thin_delegate::derive_delegate]
impl Hello for Hoge {}

fn main() {
    let hoge = Hoge::A("a".to_string());
    assert_eq!(hoge.hello(), "hello, a");

    let hoge = Hoge::B('b');
    assert_eq!(hoge.hello(), "hello, b");
}
