// Compare with pass_fill_missing_functions_in_impl.rs
//
// You need to fill associated consts. thin_delegate doesn't automatically fill them because there
// is no natural choise in general, e.g. enum case.

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

#[thin_delegate::fill_delegate]
impl Hello for Hoge {
    type Return = String;

    // const NEED_TO_FILL: &'static str = "Hoge";

    fn override_(&self) -> Self::Return {
        self.0.override_().to_uppercase()
    }
}

fn main() {}
