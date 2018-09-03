#[macro_use]
extern crate gdnative;

#[no_mangle]
pub extern "C" fn run_tests(
    _data: *mut gdnative::libc::c_void,
    _args: *mut gdnative::sys::godot_array
) -> gdnative::sys::godot_variant {

    let mut status = true;
    status &= gdnative::test_string();

    status &= gdnative::test_dictionary();
    // status &= gdnative::test_dictionary_clone_clear();

    status &= gdnative::test_array();
    // status &= gdnative::test_array_clone_clear();

    status &= gdnative::test_variant_nil();
    status &= gdnative::test_variant_i64();

    status &= gdnative::test_vector2_variants();

    status &= gdnative::test_vector3_variants();

    status &= test_constructor();

    gdnative::Variant::from_bool(status).forget()
}

fn test_constructor() -> bool {
    println!(" -- test_constructor");

    use gdnative::{GDNativeLibrary, Path2D};

    // Just create an object and call a method as a sanity check for the
    // generated constructors.
    let lib = GDNativeLibrary::new();
    let _ = lib.is_singleton();

    let path = Path2D::new();
    let _ = unsafe { path.get_z_index() };
    unsafe { path.free(); }

    return true;
}

godot_gdnative_init!();
godot_nativescript_init!();
godot_gdnative_terminate!();
