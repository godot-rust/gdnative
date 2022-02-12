use gdnative::prelude::*;

mod hud;
mod main_scene;
mod mob;
mod player;

fn init(handle: InitHandle) {
    handle.add_class::<player::Player>();
    handle.add_class::<mob::Mob>();
    handle.add_class::<main_scene::Main>();
    handle.add_class::<hud::Hud>();
}

godot_init!(init);
