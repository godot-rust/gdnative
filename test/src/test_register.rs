use std::ops::Add;

use gdnative::export::{StaticArgs, StaticArgsMethod};
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_register_property();
    status &= test_advanced_methods();

    status
}

pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<RegisterSignal>();
    handle.add_class::<RegisterProperty>();
    handle.add_class::<AdvancedMethods>();
}

#[derive(Copy, Clone, Debug, Default)]
struct RegisterSignal;

impl NativeClass for RegisterSignal {
    type Base = Reference;
    type UserData = user_data::Aether<RegisterSignal>;
    fn class_name() -> &'static str {
        "RegisterSignal"
    }
    fn init(_owner: TRef<Reference>) -> RegisterSignal {
        RegisterSignal
    }
    fn register_properties(builder: &ClassBuilder<Self>) {
        builder
            .signal("progress")
            .with_param("amount", VariantType::I64)
            .done();
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
    fn init(_owner: TRef<Reference>) -> RegisterProperty {
        RegisterProperty { value: 42 }
    }
    fn register_properties(builder: &ClassBuilder<Self>) {
        builder
            .property("value")
            .with_default(42)
            .with_setter(RegisterProperty::set_value)
            .with_getter(RegisterProperty::get_value)
            .done();
    }
}

#[methods]
impl RegisterProperty {
    #[export]
    fn set_value(&mut self, _owner: TRef<Reference>, value: i64) {
        self.value = value;
    }

    #[export]
    fn get_value(&self, _owner: TRef<Reference>) -> i64 {
        self.value
    }
}

fn test_register_property() -> bool {
    println!(" -- test_register_property");

    let ok = std::panic::catch_unwind(|| {
        let obj = RegisterProperty::new_instance();

        let base = obj.into_base();

        assert_eq!(Some(42), unsafe { base.call("get_value", &[]).to() });

        base.set("value", 54.to_variant());

        assert_eq!(Some(54), unsafe { base.call("get_value", &[]).to() });

        unsafe { base.call("set_value", &[4242.to_variant()]) };

        assert_eq!(Some(4242), unsafe { base.call("get_value", &[]).to() });
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_register_property failed");
    }

    ok
}

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

fn test_advanced_methods() -> bool {
    println!(" -- test_advanced_methods");

    let ok = std::panic::catch_unwind(|| {
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
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_advanced_methods failed");
    }

    ok
}
