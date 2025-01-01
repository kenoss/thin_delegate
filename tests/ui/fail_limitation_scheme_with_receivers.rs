#[thin_delegate::register]
pub trait Hello {
    fn hello_ref(&self, prefix: &str) -> String;
    fn hello_ref_mut(&mut self, prefix: &str) -> String;
    fn hello_consume(self, prefix: &str) -> String;
}

impl Hello for usize {
    fn hello_ref(&self, prefix: &str) -> String {
        format!("{prefix}, {self}")
    }

    fn hello_ref_mut(&mut self, prefix: &str) -> String {
        format!("{prefix}, {self}")
    }

    fn hello_consume(self, prefix: &str) -> String {
        format!("{prefix}, {self}")
    }
}

#[thin_delegate::register]
struct Hoge(String);

impl Hoge {
    fn key(&self) -> usize {
        self.0.len()
    }
}

// `#[thin_delegate::fill_delegate]` can't handle consuming receiver with `scheme` argument.
// You need to implement such functions manually.
//
// See also pass_scheme_with_receivers.rs.
#[thin_delegate::fill_delegate(scheme = |f| f(&mut self.key()))]
impl Hello for Hoge {}

fn main() {}
