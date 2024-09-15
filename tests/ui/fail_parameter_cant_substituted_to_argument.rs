#[thin_delegate::register]
trait Hello<'a, T> {
    fn hello(&self) -> &'a T;
}

#[thin_delegate::register]
struct Hoge;

#[thin_delegate::derive_delegate]
impl Hello<String, String> for Hoge {}

fn main() {}
