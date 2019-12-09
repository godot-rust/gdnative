#[macro_use]
extern crate gdnative as godot;
#[macro_use]
extern crate lazy_static;

extern crate euclid;
extern crate nalgebra;
extern crate specs;

pub mod singleton;
pub mod sync;
pub mod your_state;

use specs::prelude::*;

use godot::{GodotString, Variant};
use nalgebra::geometry::{Translation3, UnitQuaternion};
use nalgebra::Isometry3;
use singleton::try_get_singleton_state;
use sync::ExampleScene::ExampleCube;
use sync::{GdSceneLoc, GodotStateSync, Pos, Template3dName};
use your_state::ExampleComponent;

#[derive(gdnative::NativeClass)]
#[inherit(gdnative::Spatial)]
struct SpecsIntegration {
    // Store the loaded scene for a very slight performance boost but mostly to show you how.
    children_spawned: u32,
    state_sync: GodotStateSync, // Tracks stuff created in Godot in the past
}

// Assume godot objects are safe to Send
unsafe impl Send for SpecsIntegration {}

// Demonstrates the usage of Specs to manage a scene. UI / interactions based on scene_create
//
//   0. Every frame, the contents of the Specs World (Components & Entities) are synced to Godot
//
//  -- This stuff is similar to scene_create
//   1. Component in Specs is created when spawn_one is called
//   2. Specs Components are deleted when remove_one is called
//   3. Find and call functions in a node (Panel)
//   4. Call functions in GDNative (from panel into spawn/remove)
//
//  Note, the same mechanism which is used to call from panel into spawn_one and remove_one can be
//   used to call other GDNative classes here in rust.

#[gdnative::methods]
impl SpecsIntegration {
    fn _init(_owner: gdnative::Spatial) -> Self {
        SpecsIntegration {
            children_spawned: 0,
            state_sync: GodotStateSync::new(),
        }
    }

    #[export]
    fn _ready(&mut self, _owner: gdnative::Spatial) {
        self.state_sync.register_scene(
            Template3dName::ExampleScenes(ExampleCube),
            "res://Child_scene.tscn",
        );
    }

    #[export]
    fn _process(&mut self, owner: gdnative::Spatial, _delta: f64) {
        self.state_sync.do_sync(owner, |_world| {
            // This is a handy place where you have access to the world right before we
            //  synchronise it. Perhaps run your own systems here?
        });
    }

    #[export]
    unsafe fn spawn_one(&mut self, mut owner: gdnative::Spatial, message: GodotString) {
        godot_print!("Called spawn_one({})", message.to_string());

        // Note - This will fail if you call try_get_singleton_state from inside other code that already
        //   holds this mutex...
        let mut your_state = try_get_singleton_state()
            .expect("It looks like you are trying to access the singleton inside a nested call or two threads.");
        let world = your_state.world_mut();

        let x = (self.children_spawned % 10) as f32;
        let z = (self.children_spawned / 10) as f32;

        world
            .create_entity()
            .with(Pos(Isometry3::from_parts(
                Translation3::new(-10.0 + x * 2.0, 0.0, -10.0 + z * 2.0),
                UnitQuaternion::identity(),
            )))
            .with(GdSceneLoc {
                parent_name: "test_cubes",
                child_name: format!("child_{}", self.children_spawned),
                template: Some(Template3dName::ExampleScenes(ExampleCube)),
            })
            .with(ExampleComponent {
                child_id: self.children_spawned,
            })
            .build();

        self.children_spawned += 1;

        update_panel(&mut owner, i64::from(self.children_spawned));
    }

    #[export]
    unsafe fn remove_one(&mut self, mut owner: gdnative::Spatial, str: GodotString) {
        godot_print!("Called remove_one({})", str.to_string());
        self.children_spawned -= 1;

        let mut your_state = try_get_singleton_state()
            .expect("It looks like you are trying to access the singleton inside a nested call or two threads.");
        let world = your_state.world_mut();

        let (entities, our_components): (Entities, ReadStorage<ExampleComponent>) =
            world.system_data();
        for (entity, component) in (&entities, &our_components).join() {
            if component.child_id == self.children_spawned {
                entities.delete(entity);
            }
        }

        update_panel(&mut owner, i64::from(self.children_spawned));
    }
}

fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<SpecsIntegration>();
}

unsafe fn update_panel(owner: &mut gdnative::Spatial, num_children: i64) {
    // Here is how we call into the panel. First we get its node (we might have saved it
    //   from earlier)
    let panel_node_opt = owner
        .get_parent()
        .and_then(|parent| parent.find_node(GodotString::from_str("Panel"), true, false));
    if let Some(panel_node) = panel_node_opt {
        // Put the Node
        let mut as_variant = Variant::from_object(&panel_node);
        match as_variant.call(
            &GodotString::from_str("set_num_children"),
            &[Variant::from_u64(num_children as u64)],
        ) {
            Ok(_) => godot_print!("Called Panel OK."),
            Err(_) => godot_print!("Error calling Panel"),
        }
    } else {
        godot_print!("Could not find panel node");
    }
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
