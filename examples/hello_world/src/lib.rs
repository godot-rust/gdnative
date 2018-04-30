#[macro_use]
extern crate gdnative as godot;

godot_class! {
	class HelloWorld: godot::Node {
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

godot_init! {
	HelloWorld
}