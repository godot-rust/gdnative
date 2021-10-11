use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_variant_call_args();

    status
}

pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<VariantCallArgs>();
}

struct VariantCallArgs;

impl NativeClass for VariantCallArgs {
    type Base = Reference;
    type UserData = user_data::MutexData<VariantCallArgs>;
    fn class_name() -> &'static str {
        "VariantCallArgs"
    }
    fn init(_owner: TRef<Reference>) -> VariantCallArgs {
        VariantCallArgs
    }
    fn register_properties(_builder: &ClassBuilder<Self>) {}
}

#[methods]
impl VariantCallArgs {
    #[export]
    fn zero(&mut self, _owner: &Reference) -> i32 {
        42
    }

    #[export]
    fn one(&mut self, _owner: &Reference, a: i32) -> i32 {
        a * 42
    }

    #[export]
    fn two(&mut self, _owner: &Reference, a: i32, b: i32) -> i32 {
        a * 42 + b
    }

    #[export]
    fn three(&mut self, _owner: &Reference, a: i32, b: i32, c: i32) -> i32 {
        a * 42 + b * c
    }
}

fn test_variant_call_args() -> bool {
    println!(" -- test_variant_call_args");

    let ok = std::panic::catch_unwind(|| {
        let obj = Instance::<VariantCallArgs, _>::new();

        let mut base = obj.into_base().into_shared().to_variant();

        assert_eq!(Some(42), base.call("zero", &[]).unwrap().try_to_i64());

        assert_eq!(
            Some(126),
            base.call("one", &[Variant::from_i64(3),])
                .unwrap()
                .try_to_i64()
        );

        assert_eq!(
            Some(-10),
            base.call("two", &[Variant::from_i64(-1), Variant::from_i64(32),])
                .unwrap()
                .try_to_i64()
        );

        assert_eq!(
            Some(-52),
            base.call(
                "three",
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
