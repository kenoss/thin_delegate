#[test]
fn ui_test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass_*.rs");
    t.compile_fail("tests/ui/fail*.rs");
    #[cfg(feature = "test_smithay")]
    {
        t.pass("tests/ui/smithay/pass_*.rs");
        t.compile_fail("tests/ui/smithay/fail*.rs");
    }
}
