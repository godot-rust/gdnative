use gdnative::core_types::Margin;
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_indexed_props();

    status
}

pub(crate) fn register(_handle: InitHandle) {}

crate::godot_itest! { test_indexed_props {
    let control = Control::new();

    assert_eq!(0, control.margin_top());
    assert_eq!(0, control.margin_left());

    control.set_margin_top(42);

    assert_eq!(42, control.margin_top());
    assert_eq!(42, control.margin(Margin::Top.into()) as i64);
    assert_eq!(0, control.margin_left());
    assert_eq!(0, control.margin(Margin::Left.into()) as i64);

    control.set_margin(Margin::Left.into(), 24.0);

    assert_eq!(24, control.margin_left());

    control.free();
}}
