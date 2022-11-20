use gdnative::prelude::*;

mod player;

fn init(handle: InitHandle) {
    handle.add_class::<player::Player>();
}

godot_init!(init);
