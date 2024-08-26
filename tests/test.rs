#[test]
fn ui_test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass_*.rs");
    t.compile_fail("tests/ui/fail*.rs");
    #[cfg(feature = "unstable_delegate_to")]
    {
        t.pass("tests/ui/delegate_to_pass_*.rs");
        t.compile_fail("tests/ui/delegate_to_fail*.rs");
    }
}
