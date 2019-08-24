use gdnative::*;

#[no_mangle]
pub extern "C" fn run_tests(
    _data: *mut gdnative::libc::c_void,
    _args: *mut gdnative::sys::godot_array,
) -> gdnative::sys::godot_variant {
    let mut status = true;
    status &= gdnative::test_string();

    status &= gdnative::test_dictionary();
    // status &= gdnative::test_dictionary_clone_clear();

    status &= gdnative::test_array();
    // status &= gdnative::test_array_clone_clear();

    status &= gdnative::test_variant_nil();
    status &= gdnative::test_variant_i64();

    status &= gdnative::test_vector2_variants();

    status &= gdnative::test_vector3_variants();

    status &= test_constructor();
    status &= test_underscore_method_binding();

    status &= test_rust_class_construction();

    gdnative::Variant::from_bool(status).forget()
}

fn test_constructor() -> bool {
    println!(" -- test_constructor");

    use gdnative::{FreeOnDrop, GDNativeLibrary, Path2D};

    // Just create an object and call a method as a sanity check for the
    // generated constructors.
    let lib = GDNativeLibrary::new();
    let _ = lib.is_singleton();

    unsafe {
        let path = FreeOnDrop::new(Path2D::new());
        let _ = path.get_z_index();
    }

    return true;
}

fn test_underscore_method_binding() -> bool {
    println!(" -- test_underscore_method_binding");

    let ok = std::panic::catch_unwind(|| {
        let table = gdnative::NativeScriptMethodTable::get(get_api());
        assert_ne!(0, table._new as usize);
    }).is_ok();

    if !ok {
        godot_error!("   !! Test test_underscore_method_binding failed");
    }

    ok
}

struct Foo(i64);

impl NativeClass for Foo {
    type Base = Reference;
    fn class_name() -> &'static str {
        "Foo"
    }
    fn init(_owner: Reference) -> Foo {
        Foo(42)
    }
    fn register_properties(_builder: &init::ClassBuilder<Self>) {
    }
}

#[methods]
impl Foo {
    #[export]
    fn answer(&mut self, _owner: Reference) -> i64 {
        self.0
    }
}

fn test_rust_class_construction() -> bool {
    println!(" -- test_rust_class_construction");

    let ok = std::panic::catch_unwind(|| {
        let foo = Instance::<Foo>::new();
        assert_eq!(42, foo.apply(|foo, owner| {
            foo.answer(owner)
        }));
        assert_eq!(Some(42), unsafe { foo.into_base().call("answer".into(), &[]) }.try_to_i64());
    }).is_ok();

    if !ok {
        godot_error!("   !! Test test_rust_class_construction failed");
    }

    ok
}

fn init(handle: init::InitHandle) {
    handle.add_class::<Foo>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
