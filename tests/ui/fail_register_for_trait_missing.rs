#[thin_delegate::register]
enum Hoge {
    A(String),
    B(char),
}

#[thin_delegate::derive_delegate]
impl ToString for Hoge {}

fn main() {}
