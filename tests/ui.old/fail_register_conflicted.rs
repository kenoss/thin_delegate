mod external {
    #[thin_delegate::register(external::Hello)]
    pub trait Hello {
        fn hello(&self, prefix: &str) -> String;
    }
}

mod user {
    mod private_for_thin_delegate {
        #[thin_delegate::register(external::Hello)]
        pub trait Hello {
            fn hello(&self, prefix: &str) -> String;
        }
    }
}

fn main() {}
