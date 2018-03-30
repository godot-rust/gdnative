#[macro_use]
extern crate gdnative as godot;

use godot::*;

struct MyClass {
    // This field is usually needed in order to call methods of the parent
    // class, but in this specific case this never happens, so this field
    // is actually not needed here. 99% of the time you usually need this.
    // header: NativeInstanceHeader,

    elapsed_time: f64,
}

impl MyClass {
    fn new(_header: NativeInstanceHeader) -> Self {
        MyClass {
            // header: header,
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
        godot_print!("MyClass node was running for {} seconds",
            self.elapsed_time);
    }
}

godot_gdnative_init!(godot_rust_gdnative_init);

godot_gdnative_terminate!(godot_rust_gdnative_terminate);

godot_nativescript_init! {
    godot_rust_nativescript_init,
    |handle| {

        let constructor = godot_wrap_constructor!(MyClass, MyClass::new);
        let destructor  = godot_wrap_destructor!(MyClass);

        let ready_method = godot_wrap_method!(MyClass,
            fn _ready(&mut self) -> ());

        let ready_method = GodotScriptMethod {
            name: "_ready",
            method_ptr: Some(ready_method),
            attributes: GodotScriptMethodAttributes {
                rpc_mode: GodotRpcMode::Disabled
            },
            method_data: std::ptr::null_mut(),
            free_func: None
        };

        let process_method = godot_wrap_method!(MyClass,
            fn _process(&mut self, delta: f64) -> ());
        
        let exit_tree_method = godot_wrap_method!(MyClass,
            fn _exit_tree(&mut self) -> ());

        let builder = GodotScriptClassBuilder::new();
        builder
            .set_class_name("Test")
            .set_base_class_name("Node")
            .set_constructor(Some(constructor))
            .set_destructor(Some(destructor))

            .add_method_advanced(ready_method)

            .add_method("_process", process_method)
            .add_method("_exit_tree", exit_tree_method)

            .build(handle);
    }
}