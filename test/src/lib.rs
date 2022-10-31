#![allow(clippy::blacklisted_name)]
#![allow(deprecated)]

use gdnative::prelude::*;
use gdnative_core::godot_itest;

mod test_as_arg;
mod test_async;
mod test_constructor;
mod test_derive;
mod test_free_ub;
mod test_indexed_props;
mod test_map_owned;
mod test_register;
mod test_return_leak;
mod test_serde;
mod test_vararray_return;
mod test_variant_call_args;
mod test_variant_ops;

#[no_mangle]
pub extern "C" fn run_tests(
    _data: *mut gdnative::libc::c_void,
    _args: *mut gdnative::sys::godot_array,
) -> gdnative::sys::godot_variant {
    let mut status = true;
    status &= gdnative::core_types::test_string();
    status &= gdnative::core_types::test_string_name_eq();
    status &= gdnative::core_types::test_string_name_ord();

    status &= gdnative::core_types::test_dictionary();
    // status &= gdnative::test_dictionary_clone_clear();
    status &= gdnative::core_types::test_color();
    status &= gdnative::core_types::test_array();
    status &= gdnative::core_types::test_array_debug();
    // status &= gdnative::test_array_clone_clear();

    status &= gdnative::core_types::test_variant_nil();
    status &= gdnative::core_types::test_variant_i64();
    status &= gdnative::core_types::test_variant_bool();

    status &= gdnative::core_types::test_vector2_variants();

    status &= gdnative::core_types::test_vector3_variants();

    status &= gdnative::core_types::test_variant_option();
    status &= gdnative::core_types::test_variant_result();
    status &= gdnative::core_types::test_variant_hash_map();
    status &= gdnative::core_types::test_variant_hash_set();
    status &= gdnative::core_types::test_variant_vec();
    status &= gdnative::core_types::test_to_variant_iter();
    status &= gdnative::core_types::test_variant_tuple();
    status &= gdnative::core_types::test_variant_dispatch();

    status &= gdnative::core_types::test_byte_array_access();
    status &= gdnative::core_types::test_byte_array_debug();
    status &= gdnative::core_types::test_int32_array_access();
    status &= gdnative::core_types::test_int32_array_debug();
    status &= gdnative::core_types::test_float32_array_access();
    status &= gdnative::core_types::test_float32_array_debug();
    status &= gdnative::core_types::test_color_array_access();
    status &= gdnative::core_types::test_color_array_debug();
    status &= gdnative::core_types::test_string_array_access();
    status &= gdnative::core_types::test_string_array_debug();
    status &= gdnative::core_types::test_vector2_array_access();
    status &= gdnative::core_types::test_vector2_array_debug();
    status &= gdnative::core_types::test_vector3_array_access();
    status &= gdnative::core_types::test_vector3_array_debug();
    status &= gdnative::core_types::test_transform2d_behavior();

    status &= test_underscore_method_binding();
    status &= test_rust_class_construction();
    status &= test_from_instance_id();

    status &= test_as_arg::run_tests();
    status &= test_async::run_tests();
    status &= test_constructor::run_tests();
    status &= test_derive::run_tests();
    status &= test_free_ub::run_tests();
    status &= test_indexed_props::run_tests();
    status &= test_map_owned::run_tests();
    status &= test_register::run_tests();
    status &= test_return_leak::run_tests();
    status &= test_serde::run_tests();
    status &= test_vararray_return::run_tests();
    status &= test_variant_call_args::run_tests();
    status &= test_variant_ops::run_tests();

    Variant::new(status).leak()
}

godot_itest! { test_underscore_method_binding {
    let script = gdnative::api::NativeScript::new();
    let result = script._new(&[]);
    assert_eq!(Variant::nil(), result);
}}

#[derive(NativeClass)]
#[inherit(Reference)]
struct Foo(i64);

impl Foo {
    fn new(_owner: TRef<Reference>) -> Foo {
        Foo(42)
    }
}

#[derive(NativeClass)]
#[inherit(Reference)]
struct NotFoo;

impl NotFoo {
    fn new(_owner: &Reference) -> NotFoo {
        NotFoo
    }
}

#[methods]
impl Foo {
    #[method]
    fn answer(&self, #[base] _base: &Reference) -> i64 {
        self.0
    }

    #[method]
    fn choose(&self, a: GodotString, which: bool, b: GodotString) -> GodotString {
        if which {
            a
        } else {
            b
        }
    }

    #[method]
    fn choose_variant(&self, a: i32, what: Variant, b: f64) -> Variant {
        let what = what.try_to::<String>().expect("should be string");
        match what.as_str() {
            "int" => a.to_variant(),
            "float" => b.to_variant(),
            _ => panic!("should be int or float, got {:?}", what),
        }
    }
}

godot_itest! { test_rust_class_construction {
    let foo = Foo::new_instance();
    assert_eq!(Ok(42), foo.map(|foo, base| { foo.answer(&base) }));

    let base = foo.into_base();
    assert_eq!(Some(42), unsafe { base.call("answer", &[]).to() });

    let foo = Instance::<Foo, _>::try_from_base(base).expect("should be able to downcast");
    assert_eq!(Ok(42), foo.map(|foo, base| { foo.answer(&base) }));

    let base = foo.into_base();
    assert!(Instance::<NotFoo, _>::try_from_base(base).is_err());
}}

#[derive(NativeClass)]
#[inherit(Reference)]
struct OptionalArgs;

impl OptionalArgs {
    fn new(_owner: &Reference) -> Self {
        OptionalArgs
    }
}

#[methods]
impl OptionalArgs {
    #[method]
    #[allow(clippy::many_single_char_names)]
    fn opt_sum(
        &self, //
        a: i64,
        b: i64,
        #[opt] c: i64,
        #[opt] d: i64,
        #[opt] e: i64,
    ) -> i64 {
        a + b + c + d + e
    }
}

godot_itest! { test_from_instance_id {
    assert!(unsafe { Node::try_from_instance_id(22).is_none() });
    assert!(unsafe { Node::try_from_instance_id(42).is_none() });
    assert!(unsafe { Node::try_from_instance_id(503).is_none() });

    let instance_id;

    {
        let foo = unsafe { Node::new().into_shared().assume_safe() };
        foo.set_name("foo");

        instance_id = foo.get_instance_id();

        assert!(unsafe { Reference::try_from_instance_id(instance_id).is_none() });

        let reconstructed = unsafe { Node::from_instance_id(instance_id) };
        assert_eq!("foo", reconstructed.name().to_string());

        unsafe { foo.assume_unique().free() };
    }

    assert!(unsafe { Node::try_from_instance_id(instance_id).is_none() });

    let instance_id;

    {
        let foo = Reference::new().into_shared();
        let foo = unsafe { foo.assume_safe() };
        foo.set_meta("foo", "bar");

        instance_id = foo.get_instance_id();

        assert!(unsafe { Node::try_from_instance_id(instance_id).is_none() });

        // get_meta() got a new default parameter in Godot 3.5, which is a breaking change in Rust
        // So we cannot run this automated test for older Godot versions in CI
        #[cfg(not(feature = "custom-godot"))]
        {
            let reconstructed = unsafe { Reference::from_instance_id(instance_id) };
            assert_eq!(
                "bar",
                String::from_variant(&reconstructed.get_meta("foo", Variant::nil())).unwrap()
            );
        }
    }

    assert!(unsafe { Reference::try_from_instance_id(instance_id).is_none() });
}}

fn init(handle: InitHandle) {
    handle.add_class::<Foo>();
    handle.add_class::<OptionalArgs>();

    test_as_arg::register(handle);
    test_async::register(handle);
    test_constructor::register(handle);
    test_derive::register(handle);
    test_free_ub::register(handle);
    test_indexed_props::register(handle);
    test_map_owned::register(handle);
    test_register::register(handle);
    test_return_leak::register(handle);
    test_vararray_return::register(handle);
    test_variant_call_args::register(handle);
    test_variant_ops::register(handle);
}

fn terminate(_term_info: &gdnative::init::TerminateInfo) {
    gdnative::tasks::terminate_runtime();
}

gdnative::init::godot_gdnative_init!();
gdnative::init::godot_nativescript_init!(init);
gdnative::init::godot_gdnative_terminate!(terminate);
