trait Hello<'a, T> {
    fn hello(&self) -> &'a T;
}

// TODO: Make `register()` is usable for trait definition.
mod private_for_thin_delegate {
    #[thin_delegate::register(Hello)]
    trait Hello<'a, T> {
        fn hello(&self) -> &'a T;
    }
}

#[thin_delegate::derive_delegate(Hello<String, String>)]
struct Hoge;

fn main() {}
