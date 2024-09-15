#[thin_delegate::register]
pub trait Hello {
    fn hello(&mut self) -> String;
}

impl Hello for String {
    fn hello(&mut self) -> String {
        format!("hello, {self}")
    }
}

impl Hello for char {
    fn hello(&mut self) -> String {
        format!("hello, {self}")
    }
}

#[thin_delegate::register]
enum Hoge {
    A(String),
    B(char),
}

#[thin_delegate::derive_delegate]
impl Hello for Hoge {}

fn main() {
    let mut hoge = Hoge::A("a".to_string());
    assert_eq!(hoge.hello(), "hello, a");

    let mut hoge = Hoge::B('b');
    assert_eq!(hoge.hello(), "hello, b");
}
