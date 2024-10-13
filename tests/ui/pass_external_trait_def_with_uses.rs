// Compare with fail_external_trait_def_no_with_uses.rs
// See also //tests/ui/smithay/*.rs

mod external {
    pub struct Arg;

    pub trait Hello {
        fn hello(&self, arg: Arg) -> String;
    }

    pub trait Hi {
        fn hi(&self, arg: Arg) -> String;
    }

    impl Hello for String {
        fn hello(&self, _arg: Arg) -> String {
            format!("hello, {self}")
        }
    }

    impl Hi for String {
        fn hi(&self, _arg: Arg) -> String {
            format!("hi, {self}")
        }
    }
}

// Each `#[thin_delegate::derive_delegate(external_trait_def = __external_trait_def)]` will be
// expanded with uses as:
//
// ```
// mod ... {
//    use super::*;
//
//    use crate::external::Arg;
//
//    impl ...
// }
// ```
//
// It's convenient to copy&paste original definition as is.
#[thin_delegate::external_trait_def(with_uses = true)]
mod __external_trait_def {
    use crate::external::Arg;

    #[thin_delegate::register]
    pub trait Hello {
        fn hello(&self, arg: Arg) -> String;
    }

    #[thin_delegate::register]
    pub trait Hi {
        fn hi(&self, arg: Arg) -> String;
    }
}

#[thin_delegate::register]
struct Hoge(String);

#[thin_delegate::derive_delegate(external_trait_def = __external_trait_def)]
impl external::Hello for Hoge {}

#[thin_delegate::derive_delegate(external_trait_def = __external_trait_def)]
impl external::Hi for Hoge {}

fn main() {
    use external::{Arg, Hello, Hi};

    let hoge = Hoge("world".to_string());
    assert_eq!(hoge.hello(Arg), "hello, world");
    assert_eq!(hoge.hi(Arg), "hi, world");
}
