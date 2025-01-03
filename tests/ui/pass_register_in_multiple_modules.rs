// thin_delegate allows to register traits with the same name in different modules.

mod a {
    #[thin_delegate::register]
    pub trait Hello {
        fn hello(&self) -> String;
    }

    impl Hello for String {
        fn hello(&self) -> String {
            format!("hello, {self}")
        }
    }

    #[thin_delegate::register]
    struct Hoge(String);

    #[thin_delegate::fill_delegate]
    impl Hello for Hoge {}
}

mod b {
    #[thin_delegate::register]
    pub trait Hello {
        fn hello(&self) -> String;
    }

    impl Hello for String {
        fn hello(&self) -> String {
            format!("hello, {self}")
        }
    }

    #[thin_delegate::register]
    struct Hoge(String);

    #[thin_delegate::fill_delegate]
    impl Hello for Hoge {}
}

fn main() {}
