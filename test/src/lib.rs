use gdnative::*;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;

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
    status &= gdnative::test_variant_bool();

    status &= gdnative::test_vector2_variants();

    status &= gdnative::test_vector3_variants();

    status &= gdnative::test_variant_option();
    status &= gdnative::test_variant_result();
    status &= gdnative::test_to_variant_iter();

    status &= gdnative::test_byte_array_access();
    status &= gdnative::test_int32_array_access();
    status &= gdnative::test_float32_array_access();
    status &= gdnative::test_color_array_access();
    status &= gdnative::test_string_array_access();
    status &= gdnative::test_vector2_array_access();
    status &= gdnative::test_vector3_array_access();

    status &= test_constructor();
    status &= test_underscore_method_binding();
    status &= test_derive_to_variant();

    status &= test_rust_class_construction();
    status &= test_owner_free_ub();

    gdnative::Variant::from_bool(status).forget()
}

fn test_constructor() -> bool {
    println!(" -- test_constructor");

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
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_underscore_method_binding failed");
    }

    ok
}

struct Foo(i64);

impl NativeClass for Foo {
    type Base = Reference;
    type UserData = user_data::ArcData<Foo>;
    fn class_name() -> &'static str {
        "Foo"
    }
    fn init(_owner: Reference) -> Foo {
        Foo(42)
    }
    fn register_properties(_builder: &init::ClassBuilder<Self>) {}
}

#[methods]
impl Foo {
    #[export]
    fn answer(&self, _owner: Reference) -> i64 {
        self.0
    }

    #[export]
    fn choose(
        &self,
        _owner: Reference,
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
    fn choose_variant(&self, _owner: Reference, a: i32, what: Variant, b: f64) -> Variant {
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
        let foo = Instance::<Foo>::new();
        assert_eq!(Ok(42), foo.map(|foo, owner| { foo.answer(owner) }));
        assert_eq!(
            Some(42),
            unsafe { foo.into_base().call("answer".into(), &[]) }.try_to_i64()
        );
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_rust_class_construction failed");
    }

    ok
}

struct Bar(i64, Option<Arc<AtomicUsize>>);

impl NativeClass for Bar {
    type Base = Node;
    type UserData = user_data::RwLockData<Bar>;
    fn class_name() -> &'static str {
        "Bar"
    }
    fn init(_owner: Node) -> Bar {
        Bar(42, None)
    }
    fn register_properties(_builder: &init::ClassBuilder<Self>) {}
}

impl Bar {
    fn set_drop_counter(&mut self, counter: Arc<AtomicUsize>) {
        self.1 = Some(counter);
    }
}

#[methods]
impl Bar {
    #[export]
    fn free_is_not_ub(&mut self, owner: Node) -> bool {
        unsafe {
            owner.free();
        }
        assert_eq!(42, self.0, "self should not point to garbage");
        true
    }

    #[export]
    fn set_script_is_not_ub(&mut self, mut owner: Node) -> bool {
        unsafe {
            owner.set_script(None);
        }
        assert_eq!(42, self.0, "self should not point to garbage");
        true
    }
}

impl Drop for Bar {
    fn drop(&mut self) {
        let counter = self.1.take().expect("drop counter should be set");
        counter.fetch_add(1, AtomicOrdering::AcqRel);
        self.0 = 0;
    }
}

fn test_owner_free_ub() -> bool {
    println!(" -- test_owner_free_ub");

    let ok = std::panic::catch_unwind(|| {
        let drop_counter = Arc::new(AtomicUsize::new(0));

        let bar = Instance::<Bar>::new();
        unsafe {
            bar.map_mut_aliased(|bar, _| bar.set_drop_counter(drop_counter.clone()))
                .expect("lock should not fail");
            let mut base = bar.into_base();
            assert_eq!(
                Some(true),
                base.call("set_script_is_not_ub".into(), &[]).try_to_bool()
            );
            base.free();
        }

        let bar = Instance::<Bar>::new();
        unsafe {
            bar.map_mut_aliased(|bar, _| bar.set_drop_counter(drop_counter.clone()))
                .expect("lock should not fail");
            assert_eq!(
                Some(true),
                bar.into_base()
                    .call("free_is_not_ub".into(), &[])
                    .try_to_bool()
            );
        }

        // the values are eventually dropped
        assert_eq!(2, drop_counter.load(AtomicOrdering::Acquire));
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_owner_free_ub failed");
    }

    ok
}

fn test_derive_to_variant() -> bool {
    println!(" -- test_derive_to_variant");

    #[derive(Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    struct ToVar<T>
    where
        T: Associated,
    {
        foo: T::A,
        bar: T,
        baz: ToVarEnum<T::B>,
    }

    #[derive(Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    enum ToVarEnum<T> {
        Foo(T),
        Bar,
        Baz { baz: u8 },
    }

    trait Associated {
        type A;
        type B;
    }

    impl Associated for f64 {
        type A = i64;
        type B = bool;
    }

    let ok = std::panic::catch_unwind(|| {
        let data = ToVar::<f64> {
            foo: 42,
            bar: 54.0,
            baz: ToVarEnum::Foo(true),
        };
        let variant = data.to_variant();
        let dictionary = variant.try_to_dictionary().expect("should be dictionary");
        assert_eq!(Some(42), dictionary.get(&"foo".into()).try_to_i64());
        assert_eq!(Some(54.0), dictionary.get(&"bar".into()).try_to_f64());
        let enum_dict = dictionary
            .get(&"baz".into())
            .try_to_dictionary()
            .expect("should be dictionary");
        assert_eq!(Some(true), enum_dict.get(&"Foo".into()).try_to_bool());
        assert_eq!(
            Some(&data.baz),
            ToVarEnum::from_variant(&enum_dict.to_variant()).as_ref()
        );
        assert_eq!(Some(&data), ToVar::from_variant(&variant).as_ref());
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_derive_to_variant failed");
    }

    ok
}

fn init(handle: init::InitHandle) {
    handle.add_class::<Foo>();
    handle.add_class::<Bar>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
