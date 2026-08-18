#[cfg(feature = "log-safety")]
#[test]
fn log_safety_is_enforced() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/log_safety_rejects_unmarked.rs");
    tests.pass("tests/ui/log_safety_accepts_marked.rs");
}
