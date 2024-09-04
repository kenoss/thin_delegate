pub trait Hello: ToString {
    fn hello(&self) -> String;
}

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

// TODO: Make `register()` is usable for trait definition.
mod private_for_thin_delegate {
    #[thin_delegate::register(Hello)]
    pub trait Hello: ToString {
        fn hello(&self) -> String;
    }
}

#[thin_delegate::derive_delegate(Hello)]
enum Hoge {
    A(String),
    B(char),
}

impl ToString for Hoge {
    fn to_string(&self) -> String {
        match self {
            Self::A(x) => x.to_string(),
            Self::B(x) => x.to_string(),
        }
    }
}

fn main() {
    let hoge = Hoge::A("a".to_string());
    assert_eq!(hoge.hello(), "hello, a");

    let hoge = Hoge::B('b');
    assert_eq!(hoge.hello(), "hello, b");
}
