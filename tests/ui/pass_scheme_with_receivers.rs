#[thin_delegate::register]
pub trait Hello {
    fn hello_ref(&self, prefix: &str) -> String;
    fn hello_ref_mut(&mut self, prefix: &str) -> String;
    fn hello_consume(self, prefix: &str) -> String;
}

impl Hello for usize {
    fn hello_ref(&self, prefix: &str) -> String {
        format!("{prefix}, {self}")
    }

    fn hello_ref_mut(&mut self, prefix: &str) -> String {
        format!("{prefix}, {self}")
    }

    fn hello_consume(self, prefix: &str) -> String {
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

#[thin_delegate::fill_delegate(scheme = |f| f(&mut self.key()))]
impl Hello for Hoge {
    // `#[thin_delegate::fill_delegate]` can't handle consuming receiver with `scheme` argument.
    // You need to implement such functions manually.
    //
    // See also fail_limitation_scheme_with_receivers.rs.
    fn hello_consume(self, prefix: &str) -> String {
        self.key().hello_consume(prefix)
    }
}

fn main() {
    let mut hoge = Hoge("hoge".to_string());
    assert_eq!(hoge.hello_ref("hello"), "hello, 4");
    assert_eq!(hoge.hello_ref_mut("hello"), "hello, 4");
    assert_eq!(hoge.hello_consume("hello"), "hello, 4");
}
