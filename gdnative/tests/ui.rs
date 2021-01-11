#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();

    // NativeClass
    t.pass("tests/ui/derive_pass.rs");
    t.pass("tests/ui/derive_property_basic.rs");
    t.compile_fail("tests/ui/derive_fail_property_hint.rs");
    t.compile_fail("tests/ui/derive_fail_inherit.rs");
    t.compile_fail("tests/ui/derive_fail_inherit_param.rs");
    t.compile_fail("tests/ui/derive_fail_methods.rs");
    t.compile_fail("tests/ui/derive_fail_methods_param.rs");
    t.compile_fail("tests/ui/derive_fail_methods_list.rs");
    t.compile_fail("tests/ui/derive_fail_methods_missing_new.rs");
    t.compile_fail("tests/ui/derive_fail_userdata.rs");
    t.compile_fail("tests/ui/derive_fail_property_empty_hint.rs");

    // Variants
    t.pass("tests/ui/variant_pass.rs");

    // ToVariant
    t.compile_fail("tests/ui/to_variant_fail_01.rs");
    t.compile_fail("tests/ui/to_variant_fail_02.rs");
    t.compile_fail("tests/ui/to_variant_fail_03.rs");
    t.compile_fail("tests/ui/to_variant_fail_04.rs");
    t.compile_fail("tests/ui/to_variant_fail_05.rs");
    t.compile_fail("tests/ui/to_variant_fail_06.rs");
    t.compile_fail("tests/ui/to_variant_fail_07.rs");

    // FromVariant
    t.compile_fail("tests/ui/from_variant_fail_01.rs");
    t.compile_fail("tests/ui/from_variant_fail_02.rs");
    t.compile_fail("tests/ui/from_variant_fail_03.rs");
    t.compile_fail("tests/ui/from_variant_fail_04.rs");
    t.compile_fail("tests/ui/from_variant_fail_05.rs");
    t.compile_fail("tests/ui/from_variant_fail_06.rs");
    t.compile_fail("tests/ui/from_variant_fail_07.rs");
}
