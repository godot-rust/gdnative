#[macro_use]
extern crate gdnative;

mod extensions;
mod hud;
mod main_scene;
mod mob;
mod player;

fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<player::Player>();
    handle.add_class::<mob::Mob>();
    handle.add_class::<main_scene::Main>();
    handle.add_class::<hud::HUD>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
