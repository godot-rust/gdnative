use gdnative::api;
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_constructor();
    status &= test_from_class_name();

    status
}

pub(crate) fn register(_handle: InitHandle) {}

fn test_constructor() -> bool {
    println!(" -- test_constructor");

    // Just create an object and call a method as a sanity check for the
    // generated constructors.
    let lib = api::GDNativeLibrary::new();
    let _ = lib.is_singleton();

    let path = api::Path2D::new();
    let _ = path.z_index();
    path.free();

    true
}

crate::godot_itest! { test_from_class_name {
    // Since this method is restricted to Godot types, there is no way we can detect
    // here whether any invalid objects are leaked. Instead, the CI script is modified
    // to look at stdout for any reported leaks.

    let node = Ref::<Node, _>::by_class_name("Node2D").unwrap();
    assert_eq!("Node2D", node.get_class().to_string());
    let node = node.cast::<Node2D>().unwrap();
    assert_eq!("Node2D", node.get_class().to_string());
    let _ = node.position();
    node.free();

    let shader = Ref::<Reference, _>::by_class_name("Shader").unwrap();
    assert_eq!("Shader", &shader.get_class().to_string());
    let shader = shader.cast::<Shader>().unwrap();
    assert_eq!("Shader", &shader.get_class().to_string());

    let none = Ref::<Object, _>::by_class_name("Shader");
    assert!(none.is_none());

    let none = Ref::<Node2D, _>::by_class_name("Spatial");
    assert!(none.is_none());

    let none = Ref::<Shader, _>::by_class_name("AudioEffectReverb");
    assert!(none.is_none());

    let none = Ref::<Object, _>::by_class_name("ClassThatDoesNotExistProbably");
    assert!(none.is_none());
}}
