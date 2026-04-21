//! FRAME-01 verification: `#[gallery_demo]` produces clear compile errors
//! for each macro-misuse case. Error messages are stderr-pinned via trybuild;
//! a rustc diagnostic reformat may require running `TRYBUILD=overwrite`
//! (see tests/ui/README.md).

#[test]
fn compile_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_*.rs");
}
