#[thin_delegate::register]
struct Hoge(String);

#[thin_delegate::derive_delegate]
impl Hoge {}

fn main() {}
