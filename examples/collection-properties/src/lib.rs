use gdnative::api::*;
use gdnative::prelude::*;

use std::collections::{HashMap, HashSet};

// HashMap
#[derive(NativeClass, Default)]
#[inherit(Node)]
pub struct ExampleHashMapProperty {
    #[property]
    players: HashMap<i64, String>,
}

impl ExampleHashMapProperty {
    fn new(_owner: &Node) -> Self {
        Self::default()
    }
}

#[methods]
impl ExampleHashMapProperty {
    #[export]
    fn _ready(&self, _owner: &Node) {
        godot_print!("HashMap:");
        for (id, name) in &self.players {
            godot_print!("Hello, {} - {}!", id, name);
        }
    }
}

// HashSet
#[derive(NativeClass, Default)]
#[inherit(Node)]
pub struct ExampleHashSetProperty {
    #[property]
    players: HashSet<String>,
}

impl ExampleHashSetProperty {
    fn new(_owner: &Node) -> Self {
        Self::default()
    }
}

#[methods]
impl ExampleHashSetProperty {
    #[export]
    fn _ready(&self, _owner: &Node) {
        godot_print!("HashSet:");
        for name in &self.players {
            godot_print!("Hello, {}!", name);
        }
    }
}

// Vec
#[derive(NativeClass, Default)]
#[inherit(Node)]
pub struct ExampleVecProperty {
    #[property]
    players: Vec<String>,
}

impl ExampleVecProperty {
    fn new(_owner: &Node) -> Self {
        Self::default()
    }
}

#[methods]
impl ExampleVecProperty {
    #[export]
    fn _ready(&self, _owner: &Node) {
        godot_print!("Vec:");
        for name in &self.players {
            godot_print!("Hello, {}!", name);
        }
    }
}

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    // Register the new `HelloWorld` type we just declared.
    handle.add_class::<ExampleHashMapProperty>();
    handle.add_class::<ExampleHashSetProperty>();
    handle.add_class::<ExampleVecProperty>();
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);
