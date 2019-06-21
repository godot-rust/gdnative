#[macro_use]
extern crate gdnative as godot;
extern crate euclid;

use godot::init::{Property, PropertyHint, PropertyUsage};
use godot::{GodotString, Variant, ResourceLoader, PackedScene, Spatial, Int32Array};
use euclid::vec3;

#[derive(Debug, Clone, PartialEq)]
pub enum ManageErrs {
    CouldNotMakeInstance,
    RootClassNotSpatial(String),
}

#[derive(gdnative::NativeClass)]
#[inherit(gdnative::Spatial)]
struct SceneCreate{
    /// Store the loaded scene for a very slight performance boost but mostly to show you how.
    template : Option<PackedScene>,
    children_spawned : i32,
}


/// Demonstrates Scene creation, calling to/from gdscript
///
///   1. Child scene is created when spawn_1 is called
///   2. Child scenes are deleted when remove_1 is called
///   3. Find and call functions in a node (Panel)
///   4. Call functions in GDNative (from panel into spawn/remove)
///
///  Note, the same mechanism which is used to call from panel into spawn_1 and remove_1 can be
///   used to call other GDNative classes here in rust.

#[gdnative::methods]
impl SceneCreate {

    fn _init(_owner: gdnative::Spatial) -> Self {
        SceneCreate {
            template: None, // Have not loaded this template yet.
            children_spawned: -1,
        }
    }

    fn class_name() -> &'static str {
        "SceneCreate"
    }

    #[export]
    fn _ready(&mut self, _owner: gdnative::Spatial) {
        self.template = load_scene("res://Child_scene.tscn");
        match &self.template {
            Some(_scene) => godot_print!("Loaded child scene successfully!"),
            None => godot_print!("Could not load child scene. Check name."),
        }
    }

    #[export]
    fn spawn_1(&mut self, mut owner: gdnative::Spatial, str : GodotString) {

        godot_print!("Called spawn_1({})", str.to_string());

        let template = if let Some(template) = &self.template {
            template
        } else {
            godot_print!("Cannot spawn a child because we couldn't load the template scene");
            return;
        };

        // Create the scene here. Note that we are hardcoding that the parent must at least be a
        //   child of Spatial in the template argument here...
        match instance_scene::<Spatial>(template) {
            Ok(mut spatial) => {
                // Here is how you rename the child...
                self.children_spawned += 1;
                let key_str = format!("child_{}", self.children_spawned);
                unsafe{
                    spatial.set_name(GodotString::from_str(&key_str));

                    let x = (self.children_spawned % 10) as f32;
                    let z = (self.children_spawned / 10) as f32;
                    spatial.translate(vec3(-10.0 + x * 2.0, 0.0, -10.0 + z * 2.0));


                    // You need to parent the new scene under some node if you want it in the scene.
                    //   We parent it under ourselves.
                    owner.add_child(Some(spatial.to_object()), false);
                };


            },
            Err(err) => godot_print!("Could not instance Child : {:?}", err),
        }

        let num_children = unsafe {owner.get_child_count()};
        update_panel(&mut owner, num_children);
    }

    #[export]
    fn remove_1(&mut self, mut owner: gdnative::Spatial, str : GodotString) {
        godot_print!("Called remove_1({})", str.to_string());
        let num_children = unsafe {owner.get_child_count()};
        if num_children <= 0 {
            godot_print!("No children to delete");
            return;
        }

        let mut last_child = unsafe{owner.get_child(num_children - 1)};
        if let Some(mut node) = last_child {
            unsafe{node.queue_free()}
        }

        update_panel(&mut owner, num_children - 1);
    }

}

fn init(handle: godot::init::InitHandle) {
    handle.add_class::<SceneCreate>();
}

pub fn load_scene(name: &str) -> Option<PackedScene> {
    let scene = ResourceLoader::godot_singleton().load(
        GodotString::from_str(name),
        GodotString::from_str("PackedScene"), false);


    scene.and_then(|s| s.cast::<PackedScene>())
}

// Root here is needs to be the same type (or a parent type) of the node that you put in the child
//   scene as the root. For instance Spatial is used for this example.
pub fn instance_scene<Root>(scene : &PackedScene) -> Result<Root, ManageErrs>
    where Root: godot::GodotObject {
    let mut inst_option = scene.instance(0); // 0 - GEN_EDIT_STATE_DISABLED

    if let Some(mut instance) = inst_option {
        if let Some(mut instance_root) = unsafe { instance.cast::<Root>() } {
            Ok(instance_root)
        }
        else {
            Err(ManageErrs::RootClassNotSpatial(unsafe{instance.get_name()}.to_string()))

        }
    } else {
        Err(ManageErrs::CouldNotMakeInstance)
    }
}

fn update_panel(owner: &mut gdnative::Spatial, num_children : i64) {
    // Here is how we call into the panel. First we get its node (we might have saved it
    //   from earlier)
    let panel_node_opt = unsafe {
        owner.get_parent().and_then(|parent|
            parent.find_node(GodotString::from_str("Panel"), true, false)
        )};
    if let Some(panel_node) = panel_node_opt
    {
        // Put the Node
        let mut as_variant = Variant::from_object(&panel_node);
        match as_variant.call(&GodotString::from_str("set_num_children"),
                              &[Variant::from_u64(num_children as u64)]) {
            Ok(_) =>{godot_print!("Called Panel OK.")}
            Err(_)=>{godot_print!("Error calling Panel")}
        }
    }
    else {
        godot_print!("Could not find panel node");
    }
}


godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
