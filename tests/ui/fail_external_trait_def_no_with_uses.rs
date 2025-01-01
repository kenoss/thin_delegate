// Compare with fail_external_trait_def_with_uses.rs
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

#[thin_delegate::external_trait_def]
mod __external_trait_def {
    // Meaningless. `#[thin_delegate::register]` registers a token stream as is.
    use crate::external::Arg;

    #[thin_delegate::register]
    pub trait Hello {
        // OK. Replaced `Arg` with full path.
        fn hello(&self, arg: crate::external::Arg) -> String;
    }

    #[thin_delegate::register]
    pub trait Hi {
        // Not good. In `#[thin_delegate::fill_delegate]`, it will be expanded as is. So, one
        // needs to `use` for each derive. It's not convernient if a trait uses lots of types.
        fn hi(&self, arg: Arg) -> String;
    }
}

#[thin_delegate::register]
struct Hoge(String);

// OK.
#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def)]
impl external::Hello for Hoge {}

// NG.
#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def)]
impl external::Hi for Hoge {}

fn main() {
    use external::{Arg, Hello, Hi};

    let hoge = Hoge("world".to_string());
    assert_eq!(hoge.hello(Arg), "hello, world");
    assert_eq!(hoge.hi(Arg), "hi, world");
}
