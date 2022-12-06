#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();

    // NativeClass
    t.pass("tests/ui/derive_pass.rs");
    t.pass("tests/ui/derive_property_basic.rs");
    t.pass("tests/ui/derive_no_inherit.rs");
    t.compile_fail("tests/ui/derive_fail_inherit_param.rs");
    t.compile_fail("tests/ui/derive_fail_lifetime.rs");
    t.compile_fail("tests/ui/derive_fail_methods_list.rs");
    t.compile_fail("tests/ui/derive_fail_methods_missing_new.rs");
    t.compile_fail("tests/ui/derive_fail_methods_param.rs");
    t.compile_fail("tests/ui/derive_fail_methods_special_args.rs");
    t.compile_fail("tests/ui/derive_fail_methods.rs");
    t.compile_fail("tests/ui/derive_fail_property_empty_hint.rs");
    t.compile_fail("tests/ui/derive_fail_property_hint.rs");
    t.compile_fail("tests/ui/derive_fail_userdata.rs");

    // Variants
    t.pass("tests/ui/variant_pass.rs");

    // ToVariant
    t.compile_fail("tests/ui/to_variant_fail_01.rs");
    to_variant_ui_path(&t);
    t.compile_fail("tests/ui/to_variant_fail_04.rs");
    t.compile_fail("tests/ui/to_variant_fail_05.rs");
    t.compile_fail("tests/ui/to_variant_fail_06.rs");
    t.compile_fail("tests/ui/to_variant_fail_07.rs");
    t.compile_fail("tests/ui/to_variant_fail_08.rs");
    t.compile_fail("tests/ui/to_variant_fail_09.rs");

    // FromVariant
    t.compile_fail("tests/ui/from_variant_fail_01.rs");
    from_variant_ui_path(&t);
    t.compile_fail("tests/ui/from_variant_fail_04.rs");
    t.compile_fail("tests/ui/from_variant_fail_05.rs");
    t.compile_fail("tests/ui/from_variant_fail_06.rs");
    t.compile_fail("tests/ui/from_variant_fail_07.rs");
    t.compile_fail("tests/ui/from_variant_fail_08.rs");
    t.compile_fail("tests/ui/from_variant_fail_09.rs");
}

// FIXME(rust/issues/54725): Full path spans are only available on nightly as of now
#[rustversion::not(nightly)]
fn to_variant_ui_path(_t: &trybuild::TestCases) {}

// FIXME(rust/issues/54725): Full path spans are only available on nightly as of now
#[rustversion::nightly]
fn to_variant_ui_path(t: &trybuild::TestCases) {
    t.compile_fail("tests/ui/to_variant_fail_02.rs");
    t.compile_fail("tests/ui/to_variant_fail_03.rs");
}

// FIXME(rust/issues/54725): Full path spans are only available on nightly as of now
#[rustversion::not(nightly)]
fn from_variant_ui_path(_t: &trybuild::TestCases) {}

// FIXME(rust/issues/54725): Full path spans are only available on nightly as of now
#[rustversion::nightly]
fn from_variant_ui_path(t: &trybuild::TestCases) {
    t.compile_fail("tests/ui/from_variant_fail_02.rs");
    t.compile_fail("tests/ui/from_variant_fail_03.rs");
}
