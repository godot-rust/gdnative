#[macro_use]
extern crate gdnative_core as core;

#[no_mangle]
pub extern "C" fn run_tests(
    _data: *mut core::libc::c_void,
    _args: *mut core::sys::godot_array
) -> core::sys::godot_variant {

    let mut status = true;
    status &= core::test_string();

    status &= core::test_dictionary();
    // status &= core::test_dictionary_clone_clear();

    status &= core::test_array();
    // status &= core::test_array_clone_clear();

    status &= core::test_variant_nil();
    status &= core::test_variant_i64();

    status &= core::test_vector2_variants();

    status &= core::test_vector3_variants();

    status &= test_constructor();

    core::Variant::from_bool(status).forget()
}

fn test_constructor() -> bool {
    println!(" -- test_constructor");

    use core::{GDNativeLibrary, Path2D};

    // Just create an object and call a method as a sanity check for the
    // generated constructors.
    let lib = GDNativeLibrary::new();
    let _ = lib.is_singleton();

    let path = Path2D::new();
    unsafe {
        let _ =  path.get_z_index();
        path.free();
    }

    return true;
}

godot_gdnative_init!();
godot_nativescript_init!();
godot_gdnative_terminate!();
