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

    #[export]
    fn _ready(&self, base: &Node) {
        godot_print!("Vec (name):");
        for name in &self.name_vec {
            godot_print!("* {}", name);
        }

        godot_print!("\nHashMap (string -> color):");
        for (string, color) in &self.color_map {
            godot_print!("* {} -> #{}", string, color.to_html(false));
        }

        godot_print!("\nHashSet (ID):");
        for id in &self.id_set {
            godot_print!("* {}", id);
        }

        // The program has printed the contents and fulfilled its purpose, quit
        let scene_tree = base.get_tree().unwrap();
        unsafe { scene_tree.assume_safe() }.quit(0);
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<PropertyExport>();
}

godot_init!(init);
