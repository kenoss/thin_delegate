#[thin_delegate::register]
pub trait Hello {
    fn hello(&self) -> String;
}

impl Hello for String {
    fn hello(&self) -> String {
        format!("hello, {self}")
    }
}

impl Hello for char {
    fn hello(&self) -> String {
        format!("hello, {self}")
    }
}

#[thin_delegate::register]
enum HogeEnum {
    A(String),
    B(char),
}

#[thin_delegate::fill_delegate]
impl Hello for HogeEnum {}

#[thin_delegate::register]
struct HogeStruct(String);

#[thin_delegate::fill_delegate]
impl Hello for HogeStruct {}

fn main() {}
