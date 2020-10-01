#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/derive_pass.rs");
    t.pass("tests/ui/derive_property_basic.rs");
    t.compile_fail("tests/ui/derive_fail_inherit.rs");
    t.compile_fail("tests/ui/derive_fail_inherit_param.rs");
    t.compile_fail("tests/ui/derive_fail_methods.rs");
    t.compile_fail("tests/ui/derive_fail_methods_param.rs");
    t.compile_fail("tests/ui/derive_fail_methods_list.rs");
    t.compile_fail("tests/ui/derive_fail_methods_missing_new.rs");
    t.compile_fail("tests/ui/derive_fail_userdata.rs");
}
