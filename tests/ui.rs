/// Verifies every supported macro grammar family compiles in an independent crate.
#[test]
fn supported_macro_forms_compile() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/pass/*.rs");
}

/// Verifies unsupported and semantically invalid macro forms are rejected in
/// individually named compiler cases.
#[test]
fn unsupported_macro_forms_fail_to_compile() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/fail/*.rs");
}
