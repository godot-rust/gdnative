use gdnative::prelude::*;

#[derive(NativeClass, Default)]
#[inherit(Node)]
pub struct PropertyExport {
    #[property]
    name_vec: PoolArray<GodotString>,

    #[property]
    color_map: Dictionary,

    #[property]
    id_set: PoolArray<i32>,
}

#[methods]
impl PropertyExport {
    fn new(_base: &Node) -> Self {
        Self::default()
    }

    #[method]
    fn _ready(&self) {
        godot_print!("------------------------------------------------------------------");
        godot_print!("Print from Rust:");
        godot_print!("  PoolArray<GodotString>:");
        for name in &*self.name_vec.read() {
            godot_print!("  * {}", name);
        }

        godot_print!("\n  Dictionary (string -> color):");
        for (string, color) in &self.color_map {
            let color = Color::from_variant(&color).unwrap();
            godot_print!("  * {} -> #{}", string, color.to_html(false));
        }

        godot_print!("\n  PoolArray<i32>:");
        for id in &*self.id_set.read() {
            godot_print!("  * {}", id);
        }
    }
}

struct PropertyExportLibrary;

#[gdnative::init::callbacks]
impl GDNativeCallbacks for PropertyExportLibrary {
    fn nativescript_init(handle: InitHandle) {
        handle.add_class::<PropertyExport>();
    }
}
