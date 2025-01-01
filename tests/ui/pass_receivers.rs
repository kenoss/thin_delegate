// thin_delegate supports receivers `&self`, `&mut self`, `self`.

#[thin_delegate::register]
pub trait Hello {
    fn hello_ref(&self) -> String;
    fn hello_ref_mut(&mut self) -> String;
    fn hello_consume(self) -> String;
}

impl Hello for String {
    fn hello_ref(&self) -> String {
        format!("hello, {self}")
    }

    fn hello_ref_mut(&mut self) -> String {
        format!("hello, {self}")
    }

    fn hello_consume(self) -> String {
        format!("hello, {self}")
    }
}

impl Hello for char {
    fn hello_ref(&self) -> String {
        format!("hello, {self}")
    }

    fn hello_ref_mut(&mut self) -> String {
        format!("hello, {self}")
    }

    fn hello_consume(self) -> String {
        format!("hello, {self}")
    }
}

#[thin_delegate::register]
struct HogeStruct(String);

#[thin_delegate::fill_delegate]
impl Hello for HogeStruct {}

#[thin_delegate::register]
enum HogeEnum {
    A(String),
    B(char),
}

#[thin_delegate::fill_delegate]
impl Hello for HogeEnum {}

fn main() {
    let mut hoge = HogeStruct("struct".to_string());
    assert_eq!(hoge.hello_ref(), "hello, struct");
    assert_eq!(hoge.hello_ref_mut(), "hello, struct");
    assert_eq!(hoge.hello_consume(), "hello, struct");

    let mut hoge = HogeEnum::A("a".to_string());
    assert_eq!(hoge.hello_ref(), "hello, a");
    assert_eq!(hoge.hello_ref_mut(), "hello, a");
    assert_eq!(hoge.hello_consume(), "hello, a");

    let mut hoge = HogeEnum::B('b');
    assert_eq!(hoge.hello_ref(), "hello, b");
    assert_eq!(hoge.hello_ref_mut(), "hello, b");
    assert_eq!(hoge.hello_consume(), "hello, b");
}
