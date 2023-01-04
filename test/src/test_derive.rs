use std::cell::{self, Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use gdnative::export::Property;
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_derive_to_variant();
    status &= test_derive_to_variant_repr();
    status &= test_derive_to_variant_str();
    status &= test_derive_owned_to_variant();
    status &= test_derive_nativeclass();
    status &= test_derive_nativeclass_without_constructor();
    status &= test_derive_nativeclass_without_inherit();
    status &= test_derive_nativeclass_godot_attr_without_base();
    status &= test_derive_nativeclass_godot_attr_with_base();
    status &= test_derive_nativeclass_godot_attr_deref_return();
    status &= test_derive_nativeclass_godot_attr_rename_method();
    status &= test_derive_nativeclass_godot_attr_all_arguments();
    status &= test_derive_nativeclass_with_property_get_set();
    status &= test_derive_nativeclass_property_with_only_getter();

    status
}

#[cfg(not(feature = "no-manual-register"))]
pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<MinimalDerive>();
    handle.add_class::<EmplacementOnly>();
    handle.add_class::<WithoutInherit>();
    handle.add_class::<GodotAttrWithoutBase>();
    handle.add_class::<GodotAttrWithBase>();
    handle.add_class::<GodotAttrDerefReturn>();
    handle.add_class::<GodotAttrRenameMethod>();
    handle.add_class::<GodotAttrAllArguments>();
    handle.add_class::<CustomGetSet>();
    handle.add_class::<MyVec>();
}

#[cfg(feature = "no-manual-register")]
pub(crate) fn register(_handle: InitHandle) {}

// ----------------------------------------------------------------------------------------------------------------------------------------------

crate::godot_itest! { test_derive_to_variant {
    #[derive(Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    struct ToVar<T: Associated, R>
    where
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
    enum ToVarEnum<T: Bound> {
        Foo(T),
        Bar,
        Baz { baz: u8 },
    }

    #[derive(Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    struct ToVarTuple<T, R>(T::A, #[variant(skip)] R, T::B)
    where
        T: Associated,
        R: Default;

    trait Bound {}
    impl Bound for bool {}

    trait Associated {
        type A;
        type B : Bound;
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

    // Derive on uninhabitable enum results an error
    #[derive(Debug, PartialEq, FromVariant)]
    enum NoVariant {}

    let input = HashMap::from_iter([("foo", "bar")]).to_variant();
    assert_eq!(
        NoVariant::from_variant(&input),
        Err(FromVariantError::UnknownEnumVariant {
            variant: "foo".into(),
            expected: &[]
        })
    );
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

crate::godot_itest! { test_derive_to_variant_repr {
    const ANSWER: u8 = 42;

    #[derive(Copy, Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    #[variant(enum = "repr")]
    #[repr(u8)]
    enum ToVarRepr {
        A = 0,
        B,
        C,
        D = 128 - 1,
        E,
        F = ANSWER,
    }

    #[derive(Clone, Eq, PartialEq, Debug, OwnedToVariant, FromVariant)]
    #[variant(enum = "repr")]
    #[repr(u8)]
    enum ToVarReprOwned {
        A = 0,
        B,
        C,
        D = 128 - 1,
        E,
        F = ANSWER,
    }

    let variant = ToVarRepr::A.to_variant();
    assert_eq!(Some(0), variant.to::<u8>());

    let variant = ToVarRepr::B.to_variant();
    assert_eq!(Some(1), variant.to::<u8>());

    let variant = ToVarRepr::E.to_variant();
    assert_eq!(Some(128), variant.to::<u8>());

    let variant = ToVarReprOwned::A.owned_to_variant();
    assert_eq!(Some(0), variant.to::<u8>());

    let variant = ToVarReprOwned::C.owned_to_variant();
    assert_eq!(Some(2), variant.to::<u8>());

    let variant = ToVarReprOwned::F.owned_to_variant();
    assert_eq!(Some(42), variant.to::<u8>());

    assert_eq!(Some(ToVarRepr::A), Variant::new(0).to::<ToVarRepr>());
    assert_eq!(Some(ToVarRepr::B), Variant::new(1).to::<ToVarRepr>());
    assert_eq!(Some(ToVarRepr::C), Variant::new(2).to::<ToVarRepr>());
    assert_eq!(Some(ToVarRepr::D), Variant::new(127).to::<ToVarRepr>());
    assert_eq!(Some(ToVarRepr::E), Variant::new(128).to::<ToVarRepr>());
    assert_eq!(Some(ToVarRepr::F), Variant::new(42).to::<ToVarRepr>());
    assert_eq!(None, Variant::new(48).to::<ToVarRepr>());
    assert_eq!(None, Variant::new(192).to::<ToVarRepr>());
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

crate::godot_itest! { test_derive_to_variant_str {
    #[derive(Clone, Eq, PartialEq, Debug, ToVariant, FromVariant)]
    #[variant(enum = "str")]
    enum ToVarStr {
        A,
        B,
        C,
    }

    #[derive(Clone, Eq, PartialEq, Debug, OwnedToVariant, FromVariant)]
    #[variant(enum = "str")]
    enum ToVarStrOwned {
        A,
        B,
        C,
    }

    let variant = ToVarStr::A.to_variant();
    assert_eq!(Some("A"), variant.to::<String>().as_deref());

    let variant = ToVarStr::B.to_variant();
    assert_eq!(Some("B"), variant.to::<String>().as_deref());

    let variant = ToVarStr::C.to_variant();
    assert_eq!(Some("C"), variant.to::<String>().as_deref());

    let variant = ToVarStrOwned::A.owned_to_variant();
    assert_eq!(Some("A"), variant.to::<String>().as_deref());

    let variant = ToVarStrOwned::B.owned_to_variant();
    assert_eq!(Some("B"), variant.to::<String>().as_deref());

    let variant = ToVarStrOwned::C.owned_to_variant();
    assert_eq!(Some("C"), variant.to::<String>().as_deref());

    assert_eq!(Some(ToVarStr::A), Variant::new("A").to::<ToVarStr>());
    assert_eq!(Some(ToVarStr::B), Variant::new("B").to::<ToVarStr>());
    assert_eq!(Some(ToVarStr::C), Variant::new("C").to::<ToVarStr>());
    assert_eq!(None, Variant::new("").to::<ToVarStr>());
    assert_eq!(None, Variant::new("D").to::<ToVarStr>());
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

crate::godot_itest! { test_derive_owned_to_variant {
    #[derive(OwnedToVariant)]
    struct ToVar {
        arr: VariantArray<Unique>,
    }

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
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Reference)]
struct MinimalDerive(i64);

#[methods]
impl MinimalDerive {
    fn new(_owner: &Reference) -> Self {
        Self(54)
    }

    #[export] // deliberately use old attribute
    fn answer(&self, _owner: &Reference) -> i64 {
        self.0
    }
}

crate::godot_itest! { test_derive_nativeclass {
    let thing = Instance::<MinimalDerive, _>::new();
    let base: Ref<Reference, Unique> = thing.into_base();
    assert_eq!(unsafe { base.call("answer", &[]).to::<i64>() }, Some(54));
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Reference)]
#[no_constructor]
struct EmplacementOnly(i64);

#[methods]
impl EmplacementOnly {
    #[method]
    fn answer(&self, #[base] _base: &Reference) -> i64 {
        self.0
    }
}

crate::godot_itest! { test_derive_nativeclass_without_constructor {
    let foo = Instance::emplace(EmplacementOnly(54));
    assert_eq!(Ok(54), foo.map(|foo, base| { foo.answer(&base) }));

    let base = foo.into_base();
    assert_eq!(Some(54), unsafe { base.call("answer", &[]).to::<i64>() });

    let foo = Instance::<EmplacementOnly, _>::try_from_base(base)
        .expect("should be able to downcast");
    assert_eq!(Ok(54), foo.map(|foo, base| { foo.answer(&base) }));
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

#[derive(NativeClass)]
struct WithoutInherit(i64);

#[methods]
impl WithoutInherit {
    fn new(_owner: &Reference) -> Self {
        Self(54)
    }

    #[method]
    fn answer(&self) -> i64 {
        self.0
    }
}

crate::godot_itest! { test_derive_nativeclass_without_inherit {
    let thing = Instance::<WithoutInherit, _>::new();
    let base = thing.into_base();
    assert_eq!(unsafe { base.call("answer", &[]).to::<i64>() }, Some(54));
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Reference)]
struct GodotAttrWithoutBase(i64);

#[methods]
impl GodotAttrWithoutBase {
    fn new(_owner: &Reference) -> Self {
        Self(54)
    }

    #[method]
    fn answer(&self) -> i64 {
        self.0
    }
}

crate::godot_itest! { test_derive_nativeclass_godot_attr_without_base {
    let thing = Instance::<GodotAttrWithoutBase, _>::new();
    let base = thing.into_base();
    assert_eq!(unsafe { base.call("answer", &[]).to::<i64>() }, Some(54));
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Reference)]
struct GodotAttrWithBase(i64);

#[methods]
impl GodotAttrWithBase {
    fn new(_owner: &Reference) -> Self {
        Self(54)
    }

    #[method]
    fn answer(&self, #[base] _base: &Reference) -> i64 {
        self.0
    }
}

crate::godot_itest! { test_derive_nativeclass_godot_attr_with_base {
    let thing = Instance::<GodotAttrWithBase, _>::new();
    let base = thing.into_base();
    assert_eq!(unsafe { base.call("answer", &[]).to::<i64>() }, Some(54));
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Reference)]
struct GodotAttrDerefReturn(Rc<RefCell<Vec<i64>>>);

#[methods]
impl GodotAttrDerefReturn {
    fn new(_owner: &Reference) -> Self {
        let vec = Vec::from([12, 34]);
        let rc_ref = Rc::new(RefCell::new(vec));
        Self(rc_ref)
    }

    #[method(deref_return)]
    fn answer(&self) -> cell::Ref<Vec<i64>> {
        self.0.borrow()
    }
}

crate::godot_itest! { test_derive_nativeclass_godot_attr_deref_return {
    let thing = Instance::<GodotAttrDerefReturn, _>::new();
    let base = thing.into_base();

    let res = unsafe { base.call("answer", &[]).to::<Vec<i64>>() };
    assert_eq!(res, Some([12, 34].into()));
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Reference)]
struct GodotAttrRenameMethod(i64);

#[methods]
impl GodotAttrRenameMethod {
    fn new(_owner: &Reference) -> Self {
        Self(54)
    }

    #[method(name = "ask")]
    fn answer(&self) -> i64 {
        self.0
    }
}

crate::godot_itest! { test_derive_nativeclass_godot_attr_rename_method {
    let thing = Instance::<GodotAttrRenameMethod, _>::new();
    let base = thing.into_base();
    assert_eq!(unsafe { base.call("ask", &[]).to::<i64>() }, Some(54));
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Reference)]
struct GodotAttrAllArguments(Rc<RefCell<Vec<i64>>>);

#[methods]
impl GodotAttrAllArguments {
    fn new(_owner: &Reference) -> Self {
        let vec = Vec::from([12, 34]);
        let rc_ref = Rc::new(RefCell::new(vec));
        Self(rc_ref)
    }

    #[method(rpc = "disabled", name = "ask", deref_return)]
    fn answer(&self, #[base] _base: &Reference) -> cell::Ref<Vec<i64>> {
        self.0.borrow()
    }
}

crate::godot_itest! { test_derive_nativeclass_godot_attr_all_arguments {
    let thing = Instance::<GodotAttrAllArguments, _>::new();
    let base = thing.into_base();

    let res = unsafe { base.call("ask", &[]).to::<Vec<i64>>() };
    assert_eq!(res, Some([12, 34].into()));
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

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

crate::godot_itest! { test_derive_nativeclass_with_property_get_set {
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
}}

// ----------------------------------------------------------------------------------------------------------------------------------------------

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

crate::godot_itest! { test_derive_nativeclass_property_with_only_getter {
    use gdnative::export::user_data::MapMut;
    let (owner, script) = MyVec::new_instance().decouple();
    assert_eq!(u32::from_variant(&owner.get("size")).unwrap(), 0);

    script.map_mut(|script| script.add(42)).unwrap();
    assert_eq!(u32::from_variant(&owner.get("size")).unwrap(), 1);

    // check the setter doesn't work for `size`
    let _ = std::panic::catch_unwind(|| owner.set("size", 3));
    assert_eq!(u32::from_variant(&owner.get("size")).unwrap(), 1);
}}
