#[thin_delegate::external_trait_def]
mod __external_trait_def {
    #[thin_delegate::register]
    pub trait AsRef<T: ?Sized> {
        /// Converts this type into a shared reference of the (usually inferred) input type.
        #[stable(feature = "rust1", since = "1.0.0")]
        fn as_ref(&self) -> &T;
    }
}

#[thin_delegate::register]
struct Hoge(Box<dyn Fn(usize) -> usize>);

#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def)]
impl AsRef<(dyn Fn(usize) -> usize + 'static)> for Hoge {}

fn main() {}
