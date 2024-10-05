#[thin_delegate::register]
pub trait Hello {
    fn hello(&self, prefix: &str) -> String;
}

impl Hello for usize {
    fn hello(&self, prefix: &str) -> String {
        format!("{prefix}, {self}")
    }
}

#[thin_delegate::register]
struct Hoge(String);

impl Hoge {
    fn key(&self) -> usize {
        self.0.len()
    }
}

#[thin_delegate::derive_delegate(scheme = |f| f(&self.key()))]
impl Hello for Hoge {}

fn main() {
    let hoge = Hoge("hoge".to_string());
    assert_eq!(hoge.hello("hello"), "hello, 4");
}
