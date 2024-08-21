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

// TODO: Make `register()` is usable for trait definition.
mod private_for_thin_delegate {
    #[thin_delegate::register(Hello)]
    pub trait Hello {
        fn hello(&mut self) -> String;
    }
}

#[thin_delegate::derive_delegate(Hello)]
enum Hoge {
    A(String),
    B(char),
}

fn main() {
    let mut hoge = Hoge::A("a".to_string());
    assert_eq!(hoge.hello(), "hello, a");

    let mut hoge = Hoge::B('b');
    assert_eq!(hoge.hello(), "hello, b");
}
