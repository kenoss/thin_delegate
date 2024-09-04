pub trait Hello {
    fn hello(&self, prefix: &str) -> String;
}

impl Hello for String {
    fn hello(&self, prefix: &str) -> String {
        format!("{prefix}{self}")
    }
}

impl Hello for char {
    fn hello(&self, prefix: &str) -> String {
        format!("{prefix}{self}")
    }
}

// TODO: Make `register()` is usable for trait definition.
mod private_for_thin_delegate {
    #[thin_delegate::register(Hello)]
    pub trait Hello {
        fn hello(&self, prefix: &str) -> String;
    }
}

#[thin_delegate::derive_delegate(Hello)]
enum Hoge {
    A(String),
    B(char),
}

fn main() {
    let hoge = Hoge::A("a".to_string());
    assert_eq!(hoge.hello("hello, "), "hello, a");

    let hoge = Hoge::B('b');
    assert_eq!(hoge.hello("hello, "), "hello, b");
}
