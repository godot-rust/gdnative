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
    status &= gdnative::test_variant_tuple();

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

    status &= test_variant_call_args();
    status &= test_register_property();

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

struct NotFoo;

impl NativeClass for NotFoo {
    type Base = Reference;
    type UserData = user_data::ArcData<NotFoo>;
    fn class_name() -> &'static str {
        "NotFoo"
    }
    fn init(_owner: Reference) -> NotFoo {
        NotFoo
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

        let mut base = foo.into_base();
        assert_eq!(
            Some(42),
            unsafe { base.call("answer".into(), &[]) }.try_to_i64()
        );

        let foo = Instance::<Foo>::try_from_base(base).expect("should be able to downcast");
        assert_eq!(Ok(42), foo.map(|foo, owner| { foo.answer(owner) }));

        let base = foo.into_base();
        assert!(Instance::<NotFoo>::try_from_base(base).is_none());
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
        #[variant(with = "variant_with")]
        ptr: *mut (),
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

    mod variant_with {
        use gdnative::{FromVariantError, GodotString, ToVariant, Variant};

        pub fn to_variant(_ptr: &*mut ()) -> Variant {
            GodotString::from("*mut ()").to_variant()
        }

        pub fn from_variant(_variant: &Variant) -> Result<*mut (), FromVariantError> {
            Ok(std::ptr::null_mut())
        }
    }

    let ok = std::panic::catch_unwind(|| {
        let data = ToVar::<f64> {
            foo: 42,
            bar: 54.0,
            baz: ToVarEnum::Foo(true),
            ptr: std::ptr::null_mut(),
        };
        let variant = data.to_variant();
        let dictionary = variant.try_to_dictionary().expect("should be dictionary");
        assert_eq!(Some(42), dictionary.get(&"foo".into()).try_to_i64());
        assert_eq!(Some(54.0), dictionary.get(&"bar".into()).try_to_f64());
        assert_eq!(
            Some("*mut ()".into()),
            dictionary.get(&"ptr".into()).try_to_string()
        );
        let enum_dict = dictionary
            .get(&"baz".into())
            .try_to_dictionary()
            .expect("should be dictionary");
        assert_eq!(Some(true), enum_dict.get(&"Foo".into()).try_to_bool());

        assert_eq!(
            Ok(&data.baz),
            ToVarEnum::from_variant(&enum_dict.to_variant()).as_ref()
        );
        assert_eq!(Ok(&data), ToVar::from_variant(&variant).as_ref());
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_derive_to_variant failed");
    }

    ok
}

struct RegisterSignal;

impl NativeClass for RegisterSignal {
    type Base = Reference;
    type UserData = user_data::ArcData<RegisterSignal>;
    fn class_name() -> &'static str {
        "RegisterSignal"
    }
    fn init(_owner: Reference) -> RegisterSignal {
        RegisterSignal
    }
    fn register_properties(builder: &init::ClassBuilder<Self>) {
        builder.add_signal(gdnative::init::Signal {
            name: "progress",
            args: &[gdnative::init::SignalArgument {
                name: "amount",
                default: gdnative::Variant::new(),
                export_info: init::ExportInfo::new(VariantType::I64),
                usage: gdnative::init::PropertyUsage::DEFAULT,
            }],
        });
    }
}

#[methods]
impl RegisterSignal {}

struct RegisterProperty {
    value: i64,
}

impl NativeClass for RegisterProperty {
    type Base = Reference;
    type UserData = user_data::MutexData<RegisterProperty>;
    fn class_name() -> &'static str {
        "RegisterProperty"
    }
    fn init(_owner: Reference) -> RegisterProperty {
        RegisterProperty { value: 42 }
    }
    fn register_properties(builder: &init::ClassBuilder<Self>) {
        builder
            .add_property("value")
            .with_default(42)
            .with_setter(RegisterProperty::set_value)
            .with_getter(RegisterProperty::get_value)
            .done();
    }
}

#[methods]
impl RegisterProperty {
    #[export]
    fn set_value(&mut self, _owner: Reference, value: i64) {
        self.value = value;
    }

    #[export]
    fn get_value(&self, _owner: Reference) -> i64 {
        self.value
    }
}

fn test_register_property() -> bool {
    println!(" -- test_register_property");

    let ok = std::panic::catch_unwind(|| {
        let obj = Instance::<RegisterProperty>::new();

        let mut base = obj.into_base();

        unsafe {
            assert_eq!(Some(42), base.call("get_value".into(), &[]).try_to_i64());

            base.set("value".into(), 54.to_variant());

            assert_eq!(Some(54), base.call("get_value".into(), &[]).try_to_i64());

            base.call("set_value".into(), &[4242.to_variant()]);

            assert_eq!(Some(4242), base.call("get_value".into(), &[]).try_to_i64());
        }
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_register_property failed");
    }

    ok
}

struct VariantCallArgs;

impl NativeClass for VariantCallArgs {
    type Base = Reference;
    type UserData = user_data::MutexData<VariantCallArgs>;
    fn class_name() -> &'static str {
        "VariantCallArgs"
    }
    fn init(_owner: Reference) -> VariantCallArgs {
        VariantCallArgs
    }
    fn register_properties(_builder: &init::ClassBuilder<Self>) {}
}

#[methods]
impl VariantCallArgs {
    #[export]
    fn zero(&mut self, _owner: Reference) -> i32 {
        42
    }

    #[export]
    fn one(&mut self, _owner: Reference, a: i32) -> i32 {
        a * 42
    }

    #[export]
    fn two(&mut self, _owner: Reference, a: i32, b: i32) -> i32 {
        a * 42 + b
    }

    #[export]
    fn three(&mut self, _owner: Reference, a: i32, b: i32, c: i32) -> i32 {
        a * 42 + b * c
    }
}

fn test_variant_call_args() -> bool {
    println!(" -- test_variant_call_args");

    let ok = std::panic::catch_unwind(|| {
        let obj = Instance::<VariantCallArgs>::new();

        let mut base = obj.into_base().to_variant();

        assert_eq!(
            Some(42),
            base.call(&"zero".into(), &[]).unwrap().try_to_i64()
        );

        assert_eq!(
            Some(126),
            base.call(&"one".into(), &[Variant::from_i64(3),])
                .unwrap()
                .try_to_i64()
        );

        assert_eq!(
            Some(-10),
            base.call(
                &"two".into(),
                &[Variant::from_i64(-1), Variant::from_i64(32),]
            )
            .unwrap()
            .try_to_i64()
        );

        assert_eq!(
            Some(-52),
            base.call(
                &"three".into(),
                &[
                    Variant::from_i64(-2),
                    Variant::from_i64(4),
                    Variant::from_i64(8),
                ]
            )
            .unwrap()
            .try_to_i64()
        );
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_variant_call_args failed");
    }

    ok
}

fn init(handle: init::InitHandle) {
    handle.add_class::<Foo>();
    handle.add_class::<Bar>();
    handle.add_class::<RegisterSignal>();
    handle.add_class::<RegisterProperty>();
    handle.add_class::<VariantCallArgs>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
