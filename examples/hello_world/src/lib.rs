#[macro_use]
extern crate gdnative as godot;

godot_class! {
    class HelloWorld: godot::Node {
        is_tool: false;

        fields {
        }

        setup(_builder) {
        }

        constructor(header) {
            HelloWorld {
                header,
            }
        }

        export fn _ready(&mut self) {
            godot_print!("hello, world.");
        }
    }
}

fn init(handle: godot::init::InitHandle) {
    HelloWorld::register_class(handle);
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
