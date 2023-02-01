use gdnative::prelude::*;

mod hud;
mod main_scene;
mod mob;
mod player;

struct DtcLibrary;

#[gdnative::init::callbacks]
impl GDNativeCallbacks for DtcLibrary {
    fn nativescript_init(handle: InitHandle) {
        handle.add_class::<player::Player>();
        handle.add_class::<mob::Mob>();
        handle.add_class::<main_scene::Main>();
        handle.add_class::<hud::Hud>();
    }
}
