use gdnative::export::StaticallyNamed;
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
    fn nativeclass_init(_owner: TRef<Reference>) -> VariantCallArgs {
        VariantCallArgs
    }
    fn nativeclass_register_properties(_builder: &ClassBuilder<Self>) {}
}

impl StaticallyNamed for VariantCallArgs {
    const CLASS_NAME: &'static str = "VariantCallArgs";
}

#[methods]
impl VariantCallArgs {
    #[method]
    fn zero(&mut self) -> i32 {
        42
    }

    #[method]
    fn one(&mut self, a: i32) -> i32 {
        a * 42
    }

    #[method]
    fn two(&mut self, a: i32, b: i32) -> i32 {
        a * 42 + b
    }

    #[method]
    fn three(&mut self, a: i32, b: i32, c: i32) -> i32 {
        a * 42 + b * c
    }
}

crate::godot_itest! { test_variant_call_args {
    let obj = Instance::<VariantCallArgs, _>::new();

    let mut base = obj.into_base().into_shared().to_variant();

    assert_eq!(Some(42), call_i64(&mut base, "zero", &[]));

    assert_eq!(Some(126), call_i64(&mut base, "one", &[Variant::new(3)]));

    assert_eq!(
        Some(-10),
        call_i64(&mut base, "two", &[Variant::new(-1), Variant::new(32)])
    );

    assert_eq!(
        Some(-52),
        call_i64(
            &mut base,
            "three",
            &[Variant::new(-2), Variant::new(4), Variant::new(8),]
        )
    );
}}

fn call_i64(variant: &mut Variant, method: &str, args: &[Variant]) -> Option<i64> {
    let result = unsafe { variant.call(method, args) };

    result.unwrap().to()
}
