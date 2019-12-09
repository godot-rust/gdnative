use godot::*;
use specs::prelude::*;

use singleton::try_get_singleton_state;
use specs::world::Index;
use specs::{RunNow, World};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use sync::{
    GDDeleteOldSpatials, GdEntityCreation, GdEntityTransform, Template3dEntity, Template3dName,
};

pub struct GodotStateSync {
    /// Please be aware that you should only use this class from the main Godot thread.
    ///  Also note that if a spatial stored here is deleted by other code or scripts in your game
    ///  then weird things will happen
    templates: HashMap<Template3dName, Template3dEntity>,
    spatials: HashMap<Index, Spatial>,
    gd_deleter: Option<GDDeleteOldSpatials>,
}

impl GodotStateSync {
    pub fn new() -> GodotStateSync {
        GodotStateSync {
            templates: Default::default(),
            spatials: Default::default(),
            gd_deleter: None,
        }
    }

    pub fn do_sync(&mut self, owner: gdnative::Spatial, sync_func: fn(&mut World) -> ()) {
        // Get the state if the mutex is not already locked right now...
        let mut your_state = if let Some(state) = try_get_singleton_state() {
            state
        } else {
            return;
        };

        let world = your_state.world_mut();

        if self.gd_deleter.is_none() {
            self.gd_deleter = Some(GDDeleteOldSpatials::new(world));
        }

        sync_func(world);
        world.maintain();

        GdEntityCreation {
            grandparent: owner.clone(),
            templates: &self.templates,
            spatials: &mut self.spatials,
            default_parent: owner,
        }
        .run_now(world);
        GdEntityTransform {
            spatials: &mut self.spatials,
        }
        .run_now(world);

        self.gd_deleter
            .as_mut()
            .unwrap()
            .apply(world.system_data(), &mut self.spatials);
    }

    pub fn register_scene(&mut self, key: Template3dName, scene_path: &str) -> bool {
        try_load_scene(&mut self.templates, key, scene_path)
    }
}

pub fn try_load_scene<K: Hash + Eq + Debug>(
    scenes: &mut HashMap<K, Template3dEntity>,
    key: K,
    scene_path: &str,
) -> bool {
    let ship_scene = ResourceLoader::godot_singleton().load(
        GodotString::from_str(scene_path),
        GodotString::from_str("PackedScene"),
        false,
    );

    if let Some(ship_scene) = ship_scene.and_then(|s| s.cast::<PackedScene>()) {
        godot_print!("Have scene for {:?} : {}", key, scene_path);
        scenes.insert(key, Template3dEntity::Scene(ship_scene));
        true
    } else {
        godot_print!(
            "scene for {:?} : {} not found or correctly loaded!",
            key,
            scene_path
        );
        false
    }
}
