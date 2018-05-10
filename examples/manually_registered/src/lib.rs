#[macro_use]
extern crate gdnative as godot;

use godot::*;

struct MyClass {
    // This field is usually needed in order to call methods of the parent
    // class, but in this specific case this never happens, so this field
    // is actually not needed here. 99% of the time you usually need this.
    header: NativeInstanceHeader,

    elapsed_time: f64,
}

impl MyClass {
    fn new(header: NativeInstanceHeader) -> Self {
        MyClass {
            header: header,
            elapsed_time: 0.0,
        }
    }

    fn _ready(&mut self) {
        godot_print!("Hello World!");
    }

    fn _process(&mut self, delta: f64) {
        self.elapsed_time += delta;
    }

    fn _exit_tree(&mut self) {
        godot_print!(
            "MyClass node was running for {} seconds",
            self.elapsed_time
        );
    }
}

impl NativeClass for MyClass {
    fn class_name() -> &'static str {
        "MyClass"
    }

    fn get_header(&self) -> &NativeInstanceHeader {
        &self.header
    }
}

fn init(gdnative_init: init::InitHandle) {
    use godot::init::*;

    let constructor = godot_wrap_constructor!(MyClass, MyClass::new);
    let destructor  = godot_wrap_destructor!(MyClass);

    let ready_method = godot_wrap_method!(
        MyClass,
        fn _ready(&mut self) -> ()
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
        fn _process(&mut self, delta: f64) -> ()
    );

    let exit_tree_method = godot_wrap_method!(
        MyClass,
        fn _exit_tree(&mut self) -> ()
    );

    let class = gdnative_init.add_class::<MyClass>(
        ClassDescriptor {
            name: "MyClass",
            base_class: "Node",
            constructor: Some(constructor),
            destructor: Some(destructor),
        }
    );
    class.add_method_advanced(ready_method);
    class.add_method("_process", process_method);
    class.add_method("_exit_tree", exit_tree_method);
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!(godot_rust_gdnative_terminate);

