// `#[thin_delegate::fill_delegate]` doesn't correctly fill trait methods with receivers except for
// `&self`, `&mut self`, `self`, as delegating to sutch types are not trivial.
//
// For example, consider to define `hello_box(self: Box<Self>)` for a type `struct Hoge(String)`.
// `Hoge::hello_box()` receives `Box<Hoge>`, but the inner type is not boxed.

use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

#[thin_delegate::register]
pub trait Hello {
    fn hello_ref(&self) -> String;
    fn hello_ref_mut(&mut self) -> String;
    fn hello_consume(self) -> String;
    fn hello_box(self: Box<Self>) -> String;
    fn hello_rc(self: Rc<Self>) -> String;
    fn hello_arc(self: Arc<Self>) -> String;
    fn hello_pin(self: Pin<&Self>) -> String;
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

    fn hello_box(self: Box<Self>) -> String {
        format!("hello, {self}")
    }

    fn hello_rc(self: Rc<Self>) -> String {
        format!("hello, {self}")
    }

    fn hello_arc(self: Arc<Self>) -> String {
        format!("hello, {self}")
    }

    fn hello_pin(self: Pin<&Self>) -> String {
        format!("hello, {self}")
    }
}

#[thin_delegate::register]
struct HogeStruct(String);

#[thin_delegate::fill_delegate]
impl Hello for HogeStruct {}

fn main() {}
