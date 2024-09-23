#[thin_delegate::register]
trait Hello {
    type Return;

    const HAS_DEFAULT: &'static str = "HAS_DEFAULT";
    const NEED_TO_FILL: &'static str;

    // `thin_delegate` only can fill associated functions.
    fn filled(&self) -> Self::Return;
    fn override_(&self) -> Self::Return;
}

impl Hello for String {
    type Return = String;

    const NEED_TO_FILL: &'static str = "String";

    fn filled(&self) -> Self::Return {
        self.clone()
    }

    fn override_(&self) -> Self::Return {
        self.clone()
    }
}

#[thin_delegate::register]
struct Hoge(String);

#[thin_delegate::derive_delegate]
impl Hello for Hoge {
    // It can handle associated types in impl.
    //
    // You need to specify them by yourself as if you don't use `thin_delegate`.
    type Return = String;

    // It can handle associated consts in impl.
    //
    // You need to specify them by yourself as if you don't use `thin_delegate`.
    const NEED_TO_FILL: &'static str = "Hoge";

    // It can handle associated functions in impl.
    //
    // If an impl doesn't has an associated function (`filled()`), it is filled.
    // If an impl has an associated function (`override_()`), it is used.
    fn override_(&self) -> Self::Return {
        self.0.override_().to_uppercase()
    }
}

fn main() {
    let hoge = Hoge("hello, world".to_string());
    assert_eq!(hoge.filled(), "hello, world");
    assert_eq!(hoge.override_(), "HELLO, WORLD");
}
