#[macro_use]
extern crate gdnative;

#[derive(gdnative::NativeClass)]
#[inherit(gdnative::Node)]
#[user_data(gdnative::user_data::ArcData<HelloWorld>)]
struct HelloWorld;

#[gdnative::methods]
impl HelloWorld {
    fn _init(_owner: gdnative::Node) -> Self {
        HelloWorld
    }

    #[export]
    fn _ready(&self, _owner: gdnative::Node) {
        godot_print!("hello, world.")
    }
}

fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<HelloWorld>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
