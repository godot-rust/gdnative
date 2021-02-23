#![allow(clippy::blacklisted_name)]

use gdnative::prelude::*;

mod test_constructor;
mod test_derive;
mod test_free_ub;
mod test_map_owned;
mod test_register;
mod test_return_leak;
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

    status &= gdnative::core_types::dictionary::test_dictionary();
    // status &= gdnative::test_dictionary_clone_clear();

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
    status &= gdnative::core_types::test_to_variant_iter();
    status &= gdnative::core_types::test_variant_tuple();

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

    status &= test_underscore_method_binding();
    status &= test_rust_class_construction();
    status &= test_from_instance_id();

    status &= test_derive::run_tests();
    status &= test_free_ub::run_tests();
    status &= test_constructor::run_tests();
    status &= test_map_owned::run_tests();
    status &= test_register::run_tests();
    status &= test_return_leak::run_tests();
    status &= test_variant_call_args::run_tests();
    status &= test_variant_ops::run_tests();
    status &= test_vararray_return::run_tests();

    gdnative::core_types::Variant::from_bool(status).forget()
}

fn test_underscore_method_binding() -> bool {
    println!(" -- test_underscore_method_binding");

    let ok = std::panic::catch_unwind(|| {
        let script = gdnative::api::NativeScript::new();
        let result = script._new(&[]);
        assert_eq!(Variant::new(), result);
    })
    .is_ok();

    if !ok {
        gdnative::godot_error!("   !! Test test_underscore_method_binding failed");
    }

    ok
}

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
    #[export]
    fn answer(&self, _owner: &Reference) -> i64 {
        self.0
    }

    #[export]
    fn choose(
        &self,
        _owner: &Reference,
        a: GodotString,
        which: bool,
        b: GodotString,
    ) -> GodotString {
        if which {
            a
        } else {
            b
        }
    }

    #[export]
    fn choose_variant(&self, _owner: &Reference, a: i32, what: Variant, b: f64) -> Variant {
        let what = what.try_to_string().expect("should be string");
        match what.as_str() {
            "int" => a.to_variant(),
            "float" => b.to_variant(),
            _ => panic!("should be int or float, got {:?}", what),
        }
    }
}

fn test_rust_class_construction() -> bool {
    println!(" -- test_rust_class_construction");

    let ok = std::panic::catch_unwind(|| {
        let foo = Foo::new_instance();

        assert_eq!(Ok(42), foo.map(|foo, owner| { foo.answer(&*owner) }));

        let base = foo.into_base();
        assert_eq!(Some(42), unsafe { base.call("answer", &[]).try_to_i64() });

        let foo = Instance::<Foo, _>::try_from_base(base).expect("should be able to downcast");
        assert_eq!(Ok(42), foo.map(|foo, owner| { foo.answer(&*owner) }));

        let base = foo.into_base();
        assert!(Instance::<NotFoo, _>::try_from_base(base).is_err());
    })
    .is_ok();

    if !ok {
        gdnative::godot_error!("   !! Test test_rust_class_construction failed");
    }

    ok
}

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
    #[export]
    #[allow(clippy::many_single_char_names)]
    fn opt_sum(
        &self,
        _owner: &Reference,
        a: i64,
        b: i64,
        #[opt] c: i64,
        #[opt] d: i64,
        #[opt] e: i64,
    ) -> i64 {
        a + b + c + d + e
    }
}

fn test_from_instance_id() -> bool {
    println!(" -- test_from_instance_id");

    let ok = std::panic::catch_unwind(|| {
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

            let reconstructed = unsafe { Reference::from_instance_id(instance_id) };
            assert_eq!(
                "bar",
                String::from_variant(&reconstructed.get_meta("foo")).unwrap()
            );
        }

        assert!(unsafe { Reference::try_from_instance_id(instance_id).is_none() });
    })
    .is_ok();

    if !ok {
        gdnative::godot_error!("   !! Test test_from_instance_id failed");
    }

    ok
}

fn init(handle: InitHandle) {
    handle.add_class::<Foo>();
    handle.add_class::<OptionalArgs>();

    test_derive::register(handle);
    test_free_ub::register(handle);
    test_constructor::register(handle);
    test_map_owned::register(handle);
    test_register::register(handle);
    test_return_leak::register(handle);
    test_variant_call_args::register(handle);
    test_variant_ops::register(handle);
    test_vararray_return::register(handle);
}

gdnative::godot_init!(init);
