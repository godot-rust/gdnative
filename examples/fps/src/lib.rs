#[macro_use]
use gdnative::*;

mod extensions;
mod player;

fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<player::Player>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();