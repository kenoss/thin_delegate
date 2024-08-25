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

// TODO: Make `register()` is usable for trait definition.
mod private_for_thin_delegate {
    #[thin_delegate::register(Hello)]
    pub trait Hello<T> {
        fn hello(&self) -> String;
    }
}

#[thin_delegate::derive_delegate(Hello<u8, u16>)]
enum Hoge {
    A(String),
    B(char),
}

fn main() {}
