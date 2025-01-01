#[thin_delegate::register]
struct Hoge(String);

#[thin_delegate::fill_delegate]
impl Hoge {}

fn main() {}
