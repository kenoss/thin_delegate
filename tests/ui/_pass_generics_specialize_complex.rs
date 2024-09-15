// TODO: Support external crates.
#[thin_delegate::register]
pub trait AsRef<T: ?Sized> {
    /// Converts this type into a shared reference of the (usually inferred) input type.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn as_ref(&self) -> &T;
}

#[thin_delegate::register]
struct Hoge(Box<dyn Fn(usize) -> usize>);

#[thin_delegate::derive_delegate]
impl AsRef<(dyn Fn(usize) -> usize + 'static)> for Hoge {}

fn main() {}
