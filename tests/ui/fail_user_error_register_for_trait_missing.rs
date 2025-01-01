// #[thin_delegate::register]
pub trait Hello {
    fn hello(&self, a: usize) -> String;
}

#[thin_delegate::register]
enum Hoge {
    A(String),
    B(char),
}

#[thin_delegate::fill_delegate]
impl Hello for Hoge {}

fn main() {}
