#[thin_delegate::register]
pub trait Hello {
    fn hello(&self) -> String;
}

impl Hello for String {
    fn hello(&self) -> String {
        format!("hello, {self}")
    }
}

impl Hello for char {
    fn hello(&self) -> String {
        format!("hello, {self}")
    }
}

#[thin_delegate::register]
enum Hoge {
    A(String),
    B(char),
}

#[thin_delegate::fill_delegate]
impl Hello for Hoge {}

fn main() {
    let hoge = Hoge::A("a".to_string());
    assert_eq!(hoge.hello(), "hello, a");

    let hoge = Hoge::B('b');
    assert_eq!(hoge.hello(), "hello, b");
}
