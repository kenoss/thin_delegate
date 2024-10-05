// This crate doesn't ensure that this works in the future. Because the author doesn't completely
// understand why this passes.

#[thin_delegate::register]
pub trait Hello {
    fn hello(&self, a: usize) -> String;
}

impl Hello for String {
    fn hello(&self, a: usize) -> String {
        format!("{a}, {self}")
    }
}

impl Hello for char {
    fn hello(&self, a: usize) -> String {
        format!("{a}, {self}")
    }
}

#[thin_delegate::register]
enum Hoge {
    A(String),
    B(char),
}

// Name conflicted, but it compiles.
//
// It is expanded to the following, which doesn't compile if we write it in source code:
//
// ```
//     impl Hello for Hoge {
//         fn hello(&self, a: usize) -> String {
// //                      ^1
//             match self {
//                 Self::A(a) => Hello::hello(&format!("{}{}", a, a), a),
// //                      ^2                                  ^3 ^3  ^1(*)
//                 Self::B(b) => Hello::hello(b, a),
// //                                            ^3
//             }
//         }
//     }
// ```
//
// ^1 comes from a trait definition in the macro `__thin_delegate__trampoline2`.
// ^2 comes from an enum definition in the macro `__thin_delegate__trampoline2`.
// ^3 comes from `scheme`, `TokenStream` arg for `#[thin_delegate::derive_delegate]`.
//
// The author's current guess:
//
// - The resolution of ^1(*) is not triggered because it comes from a trait definition.
// - The resolution of ^3 is triggered because ^3 comes from a `TokenStream` in proc macro, and is
//   caught by ^2 and ^1 is shadowed.
#[thin_delegate::derive_delegate(scheme = |f| {
    match self {
        Self::A(a) => f(&format!("{}{}", a, a)),
        Self::B(b) => f(b),
    }
})]
impl Hello for Hoge {}

fn main() {
    let a = Hoge::A("hoge".to_string());
    assert_eq!(a.hello(42), "42, hogehoge");
}
