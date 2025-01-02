// `#[thin_delegate::fill_delegate]` fills missing trait functions that don't have default implementation.
// It allows to manually define associated items.
//
// Compare with fail_intended_limitation_associated_const_misning.rs and
// fail_intended_limitation_associated_type_missing.rs

#[thin_delegate::register]
trait Hello {
    type Return;

    const HAS_DEFAULT: &'static str = "HAS_DEFAULT";
    const NEED_TO_FILL: &'static str;

    // `thin_delegate` only can fill associated functions.
    fn filled(&self) -> Self::Return;
    fn override_(&self) -> Self::Return;
    fn skipped_if_fn_has_default_impl(&self) -> Self::Return {
        self.filled()
    }
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

#[thin_delegate::fill_delegate]
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

    // It doesn't fill `skipped_if_fn_has_default_impl()` as it has default implementation.
}

fn main() {
    let hoge = Hoge("hello, world".to_string());
    assert_eq!(hoge.filled(), "hello, world");
    assert_eq!(hoge.override_(), "HELLO, WORLD");
}
