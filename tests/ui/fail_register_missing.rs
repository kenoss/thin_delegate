#[thin_delegate::derive_delegate(ToString)]
enum Hoge {
    A(String),
    B(char),
}

fn main() {}
