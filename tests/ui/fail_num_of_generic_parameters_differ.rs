#[thin_delegate::register]
pub trait Hello<T> {
    fn hello(&self) -> String;
}

impl<T> Hello<T> for String {
    fn hello(&self) -> String {
        format!("hello, {self}")
    }
}

impl<T> Hello<T> for char {
    fn hello(&self) -> String {
        format!("hello, {self}")
    }
}

#[thin_delegate::register]
enum Hoge {
    A(String),
    B(char),
}

#[thin_delegate::derive_delegate]
impl Hello<u8, u16> for Hoge {}

fn main() {}
