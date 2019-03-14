extern crate gdnative as godot;

use godot::*;

struct MyClass {
    elapsed_time: f64,
}

impl MyClass {
    fn new() -> Self {
        MyClass {
            elapsed_time: 0.0,
        }
    }

    fn _ready(&mut self, _: Node) {
        godot_print!("Hello World!");
    }

    fn _process(&mut self, _: Node, delta: f64) {
        self.elapsed_time += delta;
    }

    fn _exit_tree(&mut self, _: Node) {
        godot_print!(
            "MyClass node was running for {} seconds",
            self.elapsed_time
        );
    }
}

impl Default for MyClass {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeClass for MyClass {
    type Base = Node;

    fn class_name() -> &'static str {
        "MyClass"
    }

    fn init(_owner: Node) -> Self {
        MyClass::new()
    }
}

impl NativeClassMethods for MyClass {
    fn register(builder: &godot::init::ClassBuilder<Self>) {

        use godot::init::*;

        let ready_method = godot_wrap_method!(
            MyClass,
            fn _ready(&mut self, _owner: Node) -> ()
        );

        let ready_method = ScriptMethod {
            name: "_ready",
            method_ptr: Some(ready_method),
            attributes: ScriptMethodAttributes {
                rpc_mode: RpcMode::Disabled
            },
            method_data: std::ptr::null_mut(),
            free_func: None
        };

        let process_method = godot_wrap_method!(
            MyClass,
            fn _process(&mut self, _owner: Node, delta: f64) -> ()
        );

        let exit_tree_method = godot_wrap_method!(
            MyClass,
            fn _exit_tree(&mut self, _owner: Node) -> ()
        );

        builder.add_method_advanced(ready_method);
        builder.add_method("_process", process_method);
        builder.add_method("_exit_tree", exit_tree_method);
    }
}

fn init(gdnative_init: init::InitHandle) {
    gdnative_init.add_class::<MyClass>();
}

godot_nativescript_init!(init as godot_rust_nativescript_init);
godot_gdnative_init!(_ as godot_rust_gdnative_init);
godot_gdnative_terminate!(_ as godot_rust_gdnative_terminate);

