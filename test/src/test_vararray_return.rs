use gdnative::api::Camera;
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_vararray_return_crash();

    status
}

pub(crate) fn register(_handle: InitHandle) {}

crate::godot_itest! { test_vararray_return_crash {
    // See https://github.com/godot-rust/godot-rust/issues/422
    let camera = Camera::new();

    camera.set_frustum(5.0, Vector2::new(1.0, 2.0), 0.0, 1.0);
    camera.get_frustum(); // this should not crash!
    camera.free();
}}
