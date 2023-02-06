use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
struct HelloWorld;

#[methods]
impl HelloWorld {
    fn new(_owner: &Node) -> Self {
        HelloWorld
    }

    #[method]
    fn _ready(&self) {
        godot_print!("hello, world.")
    }
}

struct HelloWorldLibrary;

#[gdnative::init::callbacks]
impl GDNativeCallbacks for HelloWorldLibrary {
    fn nativescript_init(handle: InitHandle) {
        handle.add_class::<HelloWorld>();
    }
}
