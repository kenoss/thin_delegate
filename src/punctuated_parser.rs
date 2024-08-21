pub(crate) struct PunctuatedParser<T, P> {
    inner: syn::punctuated::Punctuated<T, P>,
}

impl<T, P> PunctuatedParser<T, P> {
    pub fn into_inner(self) -> syn::punctuated::Punctuated<T, P> {
        self.inner
    }
}

impl<T, P> syn::parse::Parse for PunctuatedParser<T, P>
where
    T: syn::parse::Parse,
    P: syn::parse::Parse,
{
    fn parse(input: &syn::parse::ParseBuffer) -> Result<Self, syn::Error> {
        let inner = syn::punctuated::Punctuated::parse_terminated(input)?;
        Ok(Self { inner })
    }
}
