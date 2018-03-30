#[macro_use]
extern crate gdnative as godot;

godot_class! {
	class HelloWorld: godot::Node {
		fields {
		}

		setup(_builder) {
		}

		constructor(godot_info) {
			HelloWorld {
				godot_info: godot_info
			}
		}

		export fn _ready(&mut self) {
			godot_print!("hello, world.");
		}
	}
}

godot_init! {
	HelloWorld
}