#[thin_delegate::register]
trait Hello {
    fn filled(&self) -> String;
    fn override_(&self) -> String;
}

impl Hello for String {
    fn filled(&self) -> String {
        self.clone()
    }

    fn override_(&self) -> String {
        self.clone()
    }
}

#[thin_delegate::register]
struct Hoge(String);

macro_rules! gen_override {
    ($self:ident, $body:tt) => {
        fn override_(&$self) -> String $body
    }
}

#[thin_delegate::fill_delegate]
impl Hello for Hoge {
    // `thin_delegate` can't recognize associated functions generated by macros because
    // the expansion of `#[thin_delegate::fill_delegate]` is earlier than ones of
    // macros inside.
    gen_override! {self, {
        self.0.override_().to_uppercase()
    }}
}

fn main() {}
