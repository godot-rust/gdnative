use gdnative::api::Object;
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_derive_to_variant();
    status &= test_derive_owned_to_variant();
    status &= test_derive_nativeclass_with_property();
    status &= test_derive_nativeclass_with_property_before_get();
    status &= test_derive_nativeclass_with_property_after_get();
    status &= test_derive_nativeclass_with_property_before_set();
    status &= test_derive_nativeclass_with_property_after_set();

    status
}

pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<PropertyHooks>();
}

fn test_derive_to_variant() -> bool {
    println!(" -- test_derive_to_variant");

    #[derive(Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    struct ToVar<T, R>
    where
        T: Associated,
        R: Default,
    {
        foo: T::A,
        bar: T,
        baz: ToVarEnum<T::B>,
        #[variant(with = "variant_with")]
        ptr: *mut (),
        #[variant(skip)]
        skipped: R,
    }

    #[derive(Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    enum ToVarEnum<T> {
        Foo(T),
        Bar,
        Baz { baz: u8 },
    }

    #[derive(Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    struct ToVarTuple<T, R>(T::A, #[variant(skip)] R, T::B)
    where
        T: Associated,
        R: Default;

    trait Associated {
        type A;
        type B;
    }

    impl Associated for f64 {
        type A = i64;
        type B = bool;
    }

    mod variant_with {
        use gdnative::core_types::{FromVariantError, GodotString, ToVariant, Variant};

        #[allow(clippy::trivially_copy_pass_by_ref)]
        pub fn to_variant(_ptr: &*mut ()) -> Variant {
            GodotString::from("*mut ()").to_variant()
        }

        pub fn from_variant(_variant: &Variant) -> Result<*mut (), FromVariantError> {
            Ok(std::ptr::null_mut())
        }
    }

    let ok = std::panic::catch_unwind(|| {
        let data = ToVar::<f64, i128> {
            foo: 42,
            bar: 54.0,
            baz: ToVarEnum::Foo(true),
            ptr: std::ptr::null_mut(),
            skipped: 42,
        };

        let variant = data.to_variant();
        let dictionary = variant.try_to_dictionary().expect("should be dictionary");
        assert_eq!(Some(42), dictionary.get("foo").try_to_i64());
        assert_eq!(Some(54.0), dictionary.get("bar").try_to_f64());
        assert_eq!(
            Some("*mut ()".into()),
            dictionary.get("ptr").try_to_string()
        );
        assert!(!dictionary.contains("skipped"));

        let enum_dict = dictionary
            .get("baz")
            .try_to_dictionary()
            .expect("should be dictionary");
        assert_eq!(Some(true), enum_dict.get("Foo").try_to_bool());

        assert_eq!(
            Ok(ToVar::<f64, i128> {
                foo: 42,
                bar: 54.0,
                baz: ToVarEnum::Foo(true),
                ptr: std::ptr::null_mut(),
                skipped: 0,
            }),
            ToVar::from_variant(&variant)
        );

        let data = ToVarTuple::<f64, i128>(1, 2, false);
        let variant = data.to_variant();
        let tuple_array = variant.try_to_array().expect("should be array");

        assert_eq!(2, tuple_array.len());
        assert_eq!(Some(1), tuple_array.get(0).try_to_i64());
        assert_eq!(Some(false), tuple_array.get(1).try_to_bool());
        assert_eq!(
            Ok(ToVarTuple::<f64, i128>(1, 0, false)),
            ToVarTuple::from_variant(&variant)
        );
    })
    .is_ok();

    if !ok {
        gdnative::godot_error!("   !! Test test_derive_to_variant failed");
    }

    ok
}

fn test_derive_owned_to_variant() -> bool {
    println!(" -- test_derive_owned_to_variant");

    #[derive(OwnedToVariant)]
    struct ToVar {
        arr: VariantArray<Unique>,
    }

    let ok = std::panic::catch_unwind(|| {
        let data = ToVar {
            arr: [1, 2, 3].iter().collect(),
        };

        let variant = data.owned_to_variant();
        let dictionary = variant.try_to_dictionary().expect("should be dictionary");
        let array = dictionary
            .get("arr")
            .try_to_array()
            .expect("should be array");
        assert_eq!(3, array.len());
        assert_eq!(
            &[1, 2, 3],
            array
                .iter()
                .map(|v| v.try_to_i64().unwrap())
                .collect::<Vec<_>>()
                .as_slice()
        );
    })
    .is_ok();

    if !ok {
        gdnative::godot_error!("   !! Test test_derive_owned_to_variant failed");
    }

    ok
}

#[derive(gdnative::NativeClass)]
#[inherit(Node)]
struct PropertyHooks {
    #[property(before_set = "Self::before_get")]
    before_get_bool: bool,
    #[property(after_set = "Self::after_get")]
    after_get_bool: bool,
    #[property(before_set = "Self::before_set")]
    before_set_bool: bool,
    #[property(after_set = "Self::after_set")]
    after_set_bool: bool,

    pub before_get_calls: u32,
    pub after_get_calls: u32,
    pub before_set_calls: u32,
    pub after_set_calls: u32,
}

#[gdnative_derive::methods]
impl PropertyHooks {
    fn new(_owner: &Node) -> Self {
        Self {
            before_get_bool: false,
            after_get_bool: false,
            before_set_bool: false,
            after_set_bool: false,
            before_get_calls: 0,
            after_get_calls: 0,
            before_set_calls: 0,
            after_set_calls: 0,
        }
    }

    fn before_get(&mut self, _owner: TRef<Node, Shared>) {
        self.before_get_calls += 1;
    }

    fn after_get(&mut self, _owner: TRef<Node, Shared>) {
        self.after_get_calls += 1;
    }

    fn before_set(&mut self, _owner: TRef<Node, Shared>) {
        self.before_set_calls += 1;
    }

    fn after_set(&mut self, _owner: TRef<Node, Shared>) {
        self.after_set_calls += 1;
    }
}

fn test_derive_nativeclass_with_property() -> bool {
    println!(" -- test_derive_nativeclass_with_property");

    #[derive(gdnative::NativeClass)]
    #[inherit(Node)]
    struct PropertyHooks {
        #[property]
        pub simple_bool: bool,
    }
    impl PropertyHooks {
        fn new(_owner: &Node) -> Self {
            Self { simple_bool: false }
        }
    }

    let ok = std::panic::catch_unwind(|| {
        let owner = Node::new();
        let class = PropertyHooks::new(&owner);
        assert_eq!(class.simple_bool, false);
    })
    .is_ok();

    if !ok {
        gdnative::godot_error!("   !! Test test_derive_nativeclass_with_property failed");
    }

    ok
}

fn test_derive_nativeclass_with_property_hooks(
    set_function: fn(Ref<Object, Unique>) -> Ref<Object, Unique>,
    expected_counts: &[u32; 4],
) -> bool {
    std::panic::catch_unwind(|| {
        use gdnative::nativescript::user_data::Map;

        let thing = Instance::<PropertyHooks, _>::new();
        let (owner, script) = thing.decouple();
        let owner = set_function(owner.upcast::<Object>());
        script
            .map(|script| {
                assert_eq!(script.before_get_calls, expected_counts[0]);
                assert_eq!(script.after_get_calls, expected_counts[1]);
                assert_eq!(script.before_set_calls, expected_counts[2]);
                assert_eq!(script.after_set_calls, expected_counts[3]);
            })
            .unwrap();
        owner.free();
    })
    .is_ok()
}

fn test_derive_nativeclass_with_property_before_get() -> bool {
    println!(" -- test_derive_nativeclass_with_property_before_get");

    let ok = test_derive_nativeclass_with_property_hooks(
        |owner| {
            owner.set("before_get_bool", true);
            owner
        },
        &[1, 0, 0, 0],
    );

    if !ok {
        gdnative::godot_error!(
            "   !! Test test_derive_nativeclass_with_property_before_get failed"
        );
    }

    ok
}

fn test_derive_nativeclass_with_property_after_get() -> bool {
    println!(" -- test_derive_nativeclass_with_property_after_get");

    let ok = test_derive_nativeclass_with_property_hooks(
        |owner| {
            owner.set("after_get_bool", true);
            owner
        },
        &[0, 1, 0, 0],
    );

    if !ok {
        gdnative::godot_error!("   !! Test test_derive_nativeclass_with_property_after_get failed");
    }

    ok
}

fn test_derive_nativeclass_with_property_before_set() -> bool {
    println!(" -- test_derive_nativeclass_with_property_before_set");

    let ok = test_derive_nativeclass_with_property_hooks(
        |owner| {
            owner.set("before_set_bool", true);
            owner
        },
        &[0, 0, 1, 0],
    );

    if !ok {
        gdnative::godot_error!(
            "   !! Test test_derive_nativeclass_with_property_before_set failed"
        );
    }

    ok
}

fn test_derive_nativeclass_with_property_after_set() -> bool {
    println!(" -- test_derive_nativeclass_with_property_after_set");

    let ok = test_derive_nativeclass_with_property_hooks(
        |owner| {
            owner.set("after_set_bool", true);
            owner
        },
        &[0, 0, 0, 1],
    );

    if !ok {
        gdnative::godot_error!("   !! Test test_derive_nativeclass_with_property_after_set failed");
    }

    ok
}
