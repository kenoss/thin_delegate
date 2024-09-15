#[thin_delegate::register]
pub trait Hello<T, const N: usize> {
    fn hello(&self) -> [T; N];
}

impl Hello<u8, 4> for char {
    fn hello(&self) -> [u8; 4] {
        let mut buf = [0; 4];
        self.encode_utf8(&mut buf);
        buf
    }
}

#[thin_delegate::register]
struct Hoge(char);

#[thin_delegate::derive_delegate]
impl Hello<u8, 4> for Hoge {}

fn main() {
    let hoge = Hoge('あ');
    assert_eq!(&hoge.hello(), &[227, 129, 130, 0]);
    assert_eq!(&hoge.hello()[..3], 'あ'.to_string().as_bytes());
}
