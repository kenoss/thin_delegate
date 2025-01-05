#[thin_delegate::register]
trait Hello {
    fn fn_with_default_impl(&self) -> String {
        self.name()
    }

    fn name(&self) -> String;
}

impl Hello for usize {
    fn name(&self) -> String {
        "usize".to_string()
    }
}

#[thin_delegate::register]
struct DelegateImpl(usize);

#[thin_delegate::fill_delegate]
impl Hello for DelegateImpl {
    fn name(&self) -> String {
        "DelegateImpl".to_string()
    }
}

#[thin_delegate::register]
struct UseDefaultImpl(usize);

#[thin_delegate::fill_delegate(delegate_fn_with_default_impl = false)]
impl Hello for UseDefaultImpl {
    fn name(&self) -> String {
        "UseDefaultImpl".to_string()
    }
}

fn main() {
    let delegate_impl = DelegateImpl(0);
    assert_eq!(delegate_impl.fn_with_default_impl(), "usize");
    let use_default_impl = UseDefaultImpl(0);
    assert_eq!(use_default_impl.fn_with_default_impl(), "UseDefaultImpl");
}
