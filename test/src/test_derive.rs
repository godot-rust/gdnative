use std::cell::Cell;

use gdnative::export::Property;
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_derive_to_variant();
    status &= test_derive_owned_to_variant();
    status &= test_derive_nativeclass_with_property_hooks();
    status &= test_derive_nativeclass_without_constructor();
    status &= test_derive_nativeclass_with_property_get_set();
    status &= test_derive_nativeclass_property_with_only_getter();

    status
}

pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<PropertyHooks>();
    handle.add_class::<EmplacementOnly>();
    handle.add_class::<CustomGetSet>();
    handle.add_class::<MyVec>();
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
        let dictionary = variant
            .try_to::<Dictionary>()
            .expect("should be dictionary");
        assert_eq!(Some(42), dictionary.get("foo").and_then(|v| v.to::<i64>()));
        assert_eq!(
            Some(54.0),
            dictionary.get("bar").and_then(|v| v.to::<f64>())
        );
        assert_eq!(
            Some("*mut ()".into()),
            dictionary.get("ptr").and_then(|v| v.to::<String>())
        );
        assert!(!dictionary.contains("skipped"));

        let enum_dict = dictionary
            .get("baz")
            .and_then(|v| v.to::<Dictionary>())
            .expect("should be dictionary");
        assert_eq!(
            Some(true),
            enum_dict.get("Foo").and_then(|v| v.to::<bool>())
        );

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
        let tuple_array = variant.to::<VariantArray>().expect("should be array");

        assert_eq!(2, tuple_array.len());
        assert_eq!(Some(1), tuple_array.get(0).to::<i64>());
        assert_eq!(Some(false), tuple_array.get(1).to::<bool>());
        assert_eq!(
            Ok(ToVarTuple::<f64, i128>(1, 0, false)),
            ToVarTuple::from_variant(&variant)
        );
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_derive_to_variant failed");
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
        let dictionary = variant.to::<Dictionary>().expect("should be dictionary");
        let array = dictionary
            .get("arr")
            .and_then(|v| v.to::<VariantArray>())
            .expect("should be array");
        assert_eq!(3, array.len());
        assert_eq!(
            &[1, 2, 3],
            array
                .iter()
                .map(|v| v.to::<i64>().unwrap())
                .collect::<Vec<_>>()
                .as_slice()
        );
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_derive_owned_to_variant failed");
    }

    ok
}

#[derive(gdnative::derive::NativeClass)]
#[inherit(Node)]
struct PropertyHooks {
    #[property(
        before_get = "Self::before_get",
        after_get = "Self::after_get",
        before_set = "Self::before_set",
        after_set = "Self::after_set"
    )]
    value: u32,

    pub before_get_called: Cell<u32>,
    pub after_get_called: Cell<u32>,
    pub before_set_value: Option<u32>,
    pub after_set_value: Option<u32>,
}

#[gdnative_derive::methods]
impl PropertyHooks {
    fn new(_owner: &Node) -> Self {
        Self {
            value: 0,
            before_get_called: Cell::new(0),
            after_get_called: Cell::new(0),
            before_set_value: None,
            after_set_value: None,
        }
    }

    fn before_get(&self, _owner: TRef<Node, Shared>) {
        assert_eq!(self.before_get_called.get(), self.after_get_called.get());
        self.before_get_called.set(self.before_get_called.get() + 1);
    }

    fn after_get(&self, _owner: TRef<Node, Shared>) {
        assert_eq!(
            self.before_get_called.get(),
            self.after_get_called.get() + 1
        );
        self.after_get_called.set(self.after_get_called.get() + 1);
    }

    fn assert_get_calls(&self, times: u32) {
        assert_eq!(times, self.before_get_called.get());
        assert_eq!(times, self.after_get_called.get());
    }

    fn before_set(&mut self, _owner: TRef<Node, Shared>) {
        self.before_set_value = Some(self.value);
    }

    fn after_set(&mut self, _owner: TRef<Node, Shared>) {
        self.after_set_value = Some(self.value);
    }

    fn reset_set_value(&mut self) {
        self.before_set_value = None;
        self.after_set_value = None;
    }
}

fn test_derive_nativeclass_with_property_hooks() -> bool {
    println!(" -- test_derive_nativeclass_with_property_hooks");

    let ok = std::panic::catch_unwind(|| {
        use gdnative::export::user_data::MapMut;

        let thing = Instance::<PropertyHooks, _>::new();
        let (owner, script) = thing.decouple();

        owner.set("value", 42);
        script
            .map_mut(|script| {
                assert_eq!(Some(0), script.before_set_value);
                assert_eq!(Some(42), script.after_set_value);
                script.reset_set_value();
            })
            .unwrap();

        script
            .map_mut(|script| {
                script.assert_get_calls(0);
            })
            .unwrap();
        assert_eq!(42, u32::from_variant(&owner.get("value")).unwrap());
        script
            .map_mut(|script| {
                script.assert_get_calls(1);
            })
            .unwrap();

        owner.set("value", 12345);
        script
            .map_mut(|script| {
                assert_eq!(Some(42), script.before_set_value);
                assert_eq!(Some(12345), script.after_set_value);
                script.reset_set_value();
            })
            .unwrap();

        owner.free();
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_derive_owned_to_variant failed");
    }

    ok
}

#[derive(NativeClass)]
#[inherit(Reference)]
#[no_constructor]
struct EmplacementOnly(i64);

#[methods]
impl EmplacementOnly {
    #[export]
    fn answer(&self, _owner: &Reference) -> i64 {
        self.0
    }
}

fn test_derive_nativeclass_without_constructor() -> bool {
    println!(" -- test_derive_nativeclass_without_constructor");

    let ok = std::panic::catch_unwind(|| {
        let foo = Instance::emplace(EmplacementOnly(54));

        assert_eq!(Ok(54), foo.map(|foo, owner| { foo.answer(&*owner) }));

        let base = foo.into_base();
        assert_eq!(Some(54), unsafe { base.call("answer", &[]).to::<i64>() });

        let foo = Instance::<EmplacementOnly, _>::try_from_base(base)
            .expect("should be able to downcast");
        assert_eq!(Ok(54), foo.map(|foo, owner| { foo.answer(&*owner) }));
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_derive_nativeclass_without_constructor failed");
    }

    ok
}

#[derive(NativeClass)]
#[inherit(Node)]
struct CustomGetSet {
    pub get_called: Cell<i32>,
    pub set_called: Cell<i32>,
    #[allow(dead_code)]
    #[property(get_ref = "Self::get_foo", set = "Self::set_foo")]
    pub foo: Property<i32>,
    pub _foo: i32,
}

#[methods]
impl CustomGetSet {
    fn new(_onwer: &Node) -> Self {
        Self {
            get_called: Cell::new(0),
            set_called: Cell::new(0),
            foo: Property::default(),
            _foo: 0,
        }
    }

    fn get_foo(&self, _owner: TRef<Node>) -> &i32 {
        self.get_called.set(self.get_called.get() + 1);
        &self._foo
    }

    fn set_foo(&mut self, _owner: TRef<Node>, value: i32) {
        self.set_called.set(self.set_called.get() + 1);
        self._foo = value;
    }
}

fn test_derive_nativeclass_with_property_get_set() -> bool {
    println!(" -- test_derive_nativeclass_with_property_get_set");
    let ok = std::panic::catch_unwind(|| {
        use gdnative::export::user_data::Map;
        let (owner, script) = CustomGetSet::new_instance().decouple();
        script
            .map(|script| {
                assert_eq!(0, script.get_called.get());
                assert_eq!(0, script.set_called.get());
            })
            .unwrap();
        owner.set("foo", 1);
        script
            .map(|script| {
                assert_eq!(0, script.get_called.get());
                assert_eq!(1, script.set_called.get());
                assert_eq!(1, script._foo);
            })
            .unwrap();
        assert_eq!(1, i32::from_variant(&owner.get("foo")).unwrap());
        script
            .map(|script| {
                assert_eq!(1, script.get_called.get());
                assert_eq!(1, script.set_called.get());
            })
            .unwrap();
        owner.free();
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_derive_nativeclass_with_property_get_set failed");
    }

    ok
}

#[derive(NativeClass)]
struct MyVec {
    vec: Vec<i32>,
    #[allow(dead_code)]
    #[property(get = "Self::get_size")]
    size: Property<u32>,
}

#[methods]
impl MyVec {
    fn new(_owner: TRef<Reference>) -> Self {
        Self {
            vec: Vec::new(),
            size: Property::default(),
        }
    }

    fn add(&mut self, val: i32) {
        self.vec.push(val);
    }

    fn get_size(&self, _owner: TRef<Reference>) -> u32 {
        self.vec.len() as u32
    }
}

fn test_derive_nativeclass_property_with_only_getter() -> bool {
    println!(" -- test_derive_nativeclass_property_with_only_getter");

    let ok = std::panic::catch_unwind(|| {
        use gdnative::export::user_data::MapMut;
        let (owner, script) = MyVec::new_instance().decouple();
        assert_eq!(u32::from_variant(&owner.get("size")).unwrap(), 0);
        script.map_mut(|script| script.add(42)).unwrap();
        assert_eq!(u32::from_variant(&owner.get("size")).unwrap(), 1);
        // check the setter doesn't work for `size`
        let _ = std::panic::catch_unwind(|| owner.set("size", 3));
        assert_eq!(u32::from_variant(&owner.get("size")).unwrap(), 1);
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_derive_nativeclass_property_with_only_getter failed");
    }

    ok
}
