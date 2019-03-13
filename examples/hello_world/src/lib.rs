#[macro_use]
extern crate gdnative as godot;

godot_class! {
    class HelloWorld: godot::Node {

        fields {
        }

        setup(_builder) {
        }

        constructor(_owner: godot::Node) {
            HelloWorld {
            }
        }

        export fn _ready(&mut self, _owner: godot::Node) {
            godot_print!("hello, world.");
        }
    }
}

fn init(handle: godot::init::InitHandle) {
    handle.add_class::<HelloWorld>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
