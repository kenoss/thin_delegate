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
//                 Self::A(a) => Hello::hello(a,    a),
// //                      ^2                 ^2(*) ^1(*)
//                 Self::B(b) => Hello::hello(b, a),
// //                                            ^3
//             }
//         }
//     }
// ```
//
// ^1 comes from a trait definition in the macro `__thin_delegate__trampoline2`.
// ^2 comes from an enum definition in the macro `__thin_delegate__trampoline2`.
//
// The author's current guess:
//
// - The resolution of ^1(*) is not triggered because it comes from a trait definition.
// - The resolution of ^2(*) is not triggered because it comes from an enum definition.
#[thin_delegate::fill_delegate]
impl Hello for Hoge {}

fn main() {
    let a = Hoge::A("hoge".to_string());
    assert_eq!(a.hello(42), "42, hoge");
}
