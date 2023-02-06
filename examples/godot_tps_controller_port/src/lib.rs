use gdnative::prelude::*;

mod player;

struct TpsLibrary;

#[gdnative::init::callbacks]
impl GDNativeCallbacks for TpsLibrary {
    fn nativescript_init(handle: InitHandle) {
        handle.add_class::<player::Player>();
    }
}
