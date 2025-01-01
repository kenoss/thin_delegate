#[thin_delegate::register]
pub trait Hello: ToString {
    fn hello(&self) -> String;
}

// Note that we can use default implementation in this case.
impl Hello for String {
    fn hello(&self) -> String {
        format!("hello, {}", &self.to_string())
    }
}

impl Hello for char {
    fn hello(&self) -> String {
        format!("hello, {}", &self.to_string())
    }
}

#[thin_delegate::register]
enum Hoge {
    A(String),
    B(char),
}

// Note that we can also derive `ToString` in this case. See pass_multiple_derive.rs.
impl ToString for Hoge {
    fn to_string(&self) -> String {
        match self {
            Self::A(x) => x.to_string(),
            Self::B(x) => x.to_string(),
        }
    }
}

#[thin_delegate::fill_delegate]
impl Hello for Hoge {}

fn main() {
    let hoge = Hoge::A("a".to_string());
    assert_eq!(hoge.hello(), "hello, a");

    let hoge = Hoge::B('b');
    assert_eq!(hoge.hello(), "hello, b");
}
