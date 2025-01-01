// #[thin_delegate::register]
pub trait Hello {
    fn hello(&self, a: usize) -> String;
}

// #[thin_delegate::register]
enum Hoge {
    A(String),
    B(char),
}

#[thin_delegate::derive_delegate]
impl Hello for Hoge {}

fn main() {}
