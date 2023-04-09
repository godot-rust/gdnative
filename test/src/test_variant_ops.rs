use gdnative::core_types::variant::{InvalidOp, VariantOperator};
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_variant_ops();

    status
}

pub(crate) fn register(_handle: InitHandle) {}

crate::godot_itest! { test_variant_ops {
    let arr = VariantArray::new();
    arr.push(&"bar".to_variant());
    arr.push(&"baz".to_variant());
    let arr = arr.into_shared().to_variant();

    assert_eq!(
        Ok(42.to_variant()),
        6.to_variant()
            .evaluate(VariantOperator::Multiply, &7.to_variant()),
    );

    assert_eq!(
        Ok(false.to_variant()),
        "foo".to_variant().evaluate(VariantOperator::In, &arr),
    );

    assert_eq!(
        Err(InvalidOp),
        "foo"
            .to_variant()
            .evaluate(VariantOperator::Multiply, &"bar".to_variant()),
    );
}}
