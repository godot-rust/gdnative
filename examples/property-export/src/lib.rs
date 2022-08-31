use gdnative::prelude::*;

use std::collections::{HashMap, HashSet};

#[derive(NativeClass, Default)]
#[inherit(Node)]
pub struct PropertyExport {
    #[property]
    name_vec: Vec<String>,

    #[property]
    color_map: HashMap<GodotString, Color>,

    #[property]
    id_set: HashSet<i64>,
}

#[methods]
impl PropertyExport {
    fn new(_base: &Node) -> Self {
        Self::default()
    }

    #[method]
    fn _ready(&self) {
        godot_print!("------------------------------------------------------------------");
        godot_print!("Print from Rust (note the unordered map/set):");
        godot_print!("  Vec (name):");
        for name in &self.name_vec {
            godot_print!("  * {}", name);
        }

        godot_print!("\n  HashMap (string -> color):");
        for (string, color) in &self.color_map {
            godot_print!("  * {} -> #{}", string, color.to_html(false));
        }

        godot_print!("\n  HashSet (ID):");
        for id in &self.id_set {
            godot_print!("  * {}", id);
        }
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<PropertyExport>();
}

godot_init!(init);
