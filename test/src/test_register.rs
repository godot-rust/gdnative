use std::error::Error;
use std::ops::Add;

use gdnative::export::{StaticArgs, StaticArgsMethod, StaticallyNamed};
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_register_property();
    status &= test_advanced_methods();
    status &= test_varargs_gets();
    status &= test_varargs_to_tuple();

    status
}

#[cfg(not(feature = "no-manual-register"))]
pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<RegisterSignal>();
    handle.add_class::<RegisterProperty>();
    handle.add_class::<AdvancedMethods>();
    handle.add_class::<VarargsGets>();
    handle.add_class::<VarargsToTuple>();
}

#[cfg(feature = "no-manual-register")]
pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<RegisterSignal>();
    handle.add_class::<RegisterProperty>();
}

#[derive(Copy, Clone, Debug, Default)]
struct RegisterSignal;

impl NativeClass for RegisterSignal {
    type Base = Reference;
    type UserData = user_data::Aether<RegisterSignal>;
    fn nativeclass_init(_owner: TRef<Reference>) -> RegisterSignal {
        RegisterSignal
    }
    fn nativeclass_register_properties(builder: &ClassBuilder<Self>) {
        builder
            .signal("progress")
            .with_param("amount", VariantType::I64)
            .done();
    }
}

impl StaticallyNamed for RegisterSignal {
    const CLASS_NAME: &'static str = "RegisterSignal";
}

#[methods]
impl RegisterSignal {}

struct RegisterProperty {
    value: i64,
}

impl NativeClass for RegisterProperty {
    type Base = Reference;
    type UserData = user_data::MutexData<RegisterProperty>;
    fn nativeclass_init(_owner: TRef<Reference>) -> RegisterProperty {
        RegisterProperty { value: 42 }
    }
    fn nativeclass_register_properties(builder: &ClassBuilder<Self>) {
        builder
            .property("value")
            .with_default(42)
            .with_setter(RegisterProperty::set_value)
            .with_getter(RegisterProperty::get_value)
            .done();
    }
}

impl StaticallyNamed for RegisterProperty {
    const CLASS_NAME: &'static str = "RegisterProperty";
}

#[methods]
impl RegisterProperty {
    // Note: the _base parameter is necessary, because registration API with_setter/with_getter matches that signature
    #[method]
    fn set_value(&mut self, #[base] _base: TRef<Reference>, value: i64) {
        self.value = value;
    }

    #[method]
    fn get_value(&self, #[base] _base: TRef<Reference>) -> i64 {
        self.value
    }
}

crate::godot_itest! { test_register_property {
    let obj = RegisterProperty::new_instance();
    let base = obj.into_base();
    assert_eq!(Some(42), unsafe { base.call("get_value", &[]).to() });

    base.set("value", 54.to_variant());
    assert_eq!(Some(54), unsafe { base.call("get_value", &[]).to() });

    unsafe { base.call("set_value", &[4242.to_variant()]) };
    assert_eq!(Some(4242), unsafe { base.call("get_value", &[]).to() });
}}

#[derive(NativeClass)]
#[inherit(Reference)]
#[register_with(register_methods)]
struct AdvancedMethods;

#[methods]
impl AdvancedMethods {
    fn new(_owner: TRef<Reference>) -> Self {
        AdvancedMethods
    }
}

#[derive(FromVarargs)]
struct AddArgs<T> {
    a: T,
    b: T,
    #[opt]
    c: Option<T>,
}

struct StatefulMixin<T> {
    d: T,
}

impl<T, C> StaticArgsMethod<C> for StatefulMixin<T>
where
    T: Copy + Add<Output = T> + Send + Sync + ToVariant + FromVariant + 'static,
    C: NativeClass,
{
    type Args = AddArgs<T>;
    fn call(&self, _this: TInstance<'_, C>, args: AddArgs<T>) -> Variant {
        let AddArgs { a, b, c } = args;

        let mut acc = a;
        acc = acc + b;
        if let Some(c) = c {
            acc = acc + c;
        }
        acc = acc + self.d;

        acc.to_variant()
    }
}

fn register_methods(builder: &ClassBuilder<AdvancedMethods>) {
    builder
        .method("add_ints", StaticArgs::new(StatefulMixin { d: 42 }))
        .done();

    builder
        .method("add_floats", StaticArgs::new(StatefulMixin { d: 4.0 }))
        .done();

    builder
        .method(
            "add_vectors",
            StaticArgs::new(StatefulMixin {
                d: Vector2::new(1.0, 2.0),
            }),
        )
        .done();
}

crate::godot_itest! { test_advanced_methods {
    let thing = Instance::<AdvancedMethods, _>::new();
    let thing = thing.base();

    assert_eq!(
        45,
        i32::from_variant(unsafe {
            &thing.call(
                "add_ints",
                &[1.to_variant(), 2.to_variant(), Variant::nil()],
            )
        })
        .unwrap()
    );

    assert_eq!(
        48,
        i32::from_variant(unsafe {
            &thing.call(
                "add_ints",
                &[1.to_variant(), 2.to_variant(), 3.to_variant()],
            )
        })
        .unwrap()
    );

    approx::assert_relative_eq!(
        6.5,
        f32::from_variant(unsafe {
            &thing.call(
                "add_floats",
                &[(5.0).to_variant(), (-2.5).to_variant(), Variant::nil()],
            )
        })
        .unwrap()
    );

    let v = Vector2::from_variant(unsafe {
        &thing.call(
            "add_vectors",
            &[
                Vector2::new(5.0, -5.0).to_variant(),
                Vector2::new(-2.5, 2.5).to_variant(),
                Variant::nil(),
            ],
        )
    })
    .unwrap();

    approx::assert_relative_eq!(3.5, v.x);
    approx::assert_relative_eq!(-0.5, v.y);
}}

#[derive(NativeClass)]
#[inherit(Reference)]
#[register_with(VarargsGets::register)]
struct VarargsGets {}

#[methods]
impl VarargsGets {
    fn new(_owner: TRef<Reference>) -> Self {
        Self {}
    }

    fn register(builder: &ClassBuilder<VarargsGets>) {
        builder.method("calc", CalcMethod).done();
    }
}

struct CalcMethod;

impl Method<VarargsGets> for CalcMethod {
    fn call(
        &self,
        _this: TInstance<'_, VarargsGets>,
        args: gdnative::export::Varargs<'_>,
    ) -> Variant {
        (|| {
            args.check_length(1..=3)?;
            let a: i64 = args.get(0)?;
            let b: i64 = args.get(1)?;
            let c: i64 = args.get_opt(2)?.unwrap_or(11);

            let ret = a * b - c;
            Ok::<Variant, Box<dyn Error>>(ret.to_variant())
        })()
        .unwrap_or_default()
    }
}

crate::godot_itest! { test_varargs_gets {
    let thing = Instance::<VarargsGets, _>::new();
    let base = thing.base();

    let args = [3_i64.to_variant(), 4_i64.to_variant(), 5_i64.to_variant()];
    assert_eq!(unsafe { base.call("calc", &args).to() }, Some(7));

    let args = [3_i64.to_variant(), 4_i64.to_variant()];
    assert_eq!(unsafe { base.call("calc", &args).to() }, Some(1));
}}

#[derive(NativeClass)]
#[inherit(Reference)]
#[register_with(VarargsToTuple::register)]
struct VarargsToTuple {}

#[methods]
impl VarargsToTuple {
    fn new(_owner: TRef<Reference>) -> Self {
        VarargsToTuple {}
    }

    fn register(builder: &ClassBuilder<VarargsToTuple>) {
        builder.method("calc", CalcMethod2).done();
    }
}

struct CalcMethod2;

impl Method<VarargsToTuple> for CalcMethod2 {
    fn call(
        &self,
        _this: TInstance<'_, VarargsToTuple>,
        args: gdnative::export::Varargs<'_>,
    ) -> Variant {
        let (a, b, c): (i64, i64, i64) =
            std::convert::TryInto::try_into(args).expect("Must be able to convert");
        let ret = a * b - c;
        ret.to_variant()
    }
}

crate::godot_itest! { test_varargs_to_tuple {
    let thing = Instance::<VarargsToTuple, _>::new();
    let base = thing.base();

    let args = [3_i64.to_variant(), 4_i64.to_variant(), 5_i64.to_variant()];
    assert_eq!(unsafe { base.call("calc", &args).to() }, Some(7));
}}
