pub trait Hello<T, const N: usize> {
    fn hello(&self) -> [T; N];
}

// TODO: Make `register()` is usable for trait definition.
mod private_for_thin_delegate {
    #[thin_delegate::register(Hello)]
    pub trait Hello<T, const N: usize> {
        fn hello(&self) -> [T; N];
    }
}

impl Hello<u8, 4> for char {
    fn hello(&self) -> [u8; 4] {
        let mut buf = [0; 4];
        self.encode_utf8(&mut buf);
        buf
    }
}

#[thin_delegate::derive_delegate(Hello<u8, 4>)]
struct Hoge(char);

fn main() {
    let hoge = Hoge('あ');
    assert_eq!(&hoge.hello(), &[227, 129, 130, 0]);
    assert_eq!(&hoge.hello()[..3], 'あ'.to_string().as_bytes());
}
