#[thin_delegate::fill_delegate(scheme = |f @ Fuga| f(&self.key()))]
impl Hello for Hoge {}

fn main() {}
