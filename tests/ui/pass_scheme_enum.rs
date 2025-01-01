#[thin_delegate::register]
pub trait Hello {
    fn hello(&self, prefix: &str) -> String;
}

impl Hello for String {
    fn hello(&self, prefix: &str) -> String {
        format!("{prefix}, {self}")
    }
}

impl Hello for char {
    fn hello(&self, prefix: &str) -> String {
        format!("{prefix}, {self}")
    }
}

#[thin_delegate::register]
enum Hoge {
    A(String),
    B(char),
}

#[thin_delegate::fill_delegate(scheme = |f| {
    match self {
        Self::A(s) => f(&format!("{s}{s}")),
        Self::B(c) => f(c),
    }
})]
impl Hello for Hoge {}

fn main() {
    let a = Hoge::A("hoge".to_string());
    assert_eq!(a.hello("hello"), "hello, hogehoge");

    let b = Hoge::B('h');
    assert_eq!(b.hello("hello"), "hello, h");
}
