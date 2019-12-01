#[macro_use]
extern crate gdnative;

mod events;
mod root;

use gdnative::init::InitHandle;

fn init(handle: InitHandle) {
    handle.add_class::<root::Root>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
