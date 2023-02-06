use gdnative::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ManageErrs {
    CouldNotMakeInstance,
    RootClassNotSpatial(String),
}

#[derive(gdnative::derive::NativeClass)]
#[inherit(Spatial)]
struct SceneCreate {
    // Store the loaded scene for a very slight performance boost but mostly to show you how.
    template: Option<Ref<PackedScene, ThreadLocal>>,
    children_spawned: u32,
}

// Demonstrates Scene creation, calling to/from gdscript
//
//   1. Child scene is created when spawn_one is called
//   2. Child scenes are deleted when remove_one is called
//   3. Find and call functions in a node (Panel)
//   4. Call functions in GDNative (from panel into spawn/remove)
//
//  Note, the same mechanism which is used to call from panel into spawn_one and remove_one can be
//   used to call other GDNative classes here in rust.

#[gdnative::derive::methods]
impl SceneCreate {
    fn new(_owner: &Spatial) -> Self {
        SceneCreate {
            template: None, // Have not loaded this template yet.
            children_spawned: 0,
        }
    }

    #[gdnative::derive::method]
    fn _ready(&mut self) {
        self.template = load_scene("res://Child_scene.tscn");
        match &self.template {
            Some(_scene) => godot_print!("Loaded child scene successfully!"),
            None => godot_print!("Could not load child scene. Check name."),
        }
    }

    #[gdnative::derive::method]
    fn spawn_one(&mut self, #[base] owner: &Spatial, message: GodotString) {
        godot_print!("Called spawn_one({})", message.to_string());

        let template = if let Some(template) = &self.template {
            template
        } else {
            godot_print!("Cannot spawn a child because we couldn't load the template scene");
            return;
        };

        // Create the scene here. Note that we are hardcoding that the parent must at least be a
        //   child of Spatial in the template argument here...
        match instance_scene::<Spatial>(template) {
            Ok(spatial) => {
                // Here is how you rename the child...
                let key_str = format!("child_{}", self.children_spawned);
                spatial.set_name(key_str);

                let x = (self.children_spawned % 10) as f32;
                let z = (self.children_spawned / 10) as f32;
                spatial.translate(Vector3::new(-10.0 + x * 2.0, 0.0, -10.0 + z * 2.0));

                // You need to parent the new scene under some node if you want it in the scene.
                //   We parent it under ourselves.
                owner.add_child(spatial.into_shared(), false);
                self.children_spawned += 1;
            }
            Err(err) => godot_print!("Could not instance Child : {:?}", err),
        }

        let num_children = owner.get_child_count();
        update_panel(owner, num_children);
    }

    #[gdnative::derive::method]
    fn remove_one(&mut self, #[base] owner: &Spatial, str: GodotString) {
        godot_print!("Called remove_one({})", str);
        let num_children = owner.get_child_count();
        if num_children <= 0 {
            godot_print!("No children to delete");
            return;
        }

        assert_eq!(self.children_spawned as i64, num_children);

        let last_child = owner.get_child(num_children - 1);
        if let Some(node) = last_child {
            unsafe {
                node.assume_unique().queue_free();
            }
            self.children_spawned -= 1;
        }

        update_panel(owner, num_children - 1);
    }
}

pub fn load_scene(path: &str) -> Option<Ref<PackedScene, ThreadLocal>> {
    let scene = load::<PackedScene>(path)?;
    let scene = unsafe { scene.assume_thread_local() };
    Some(scene)
}

/// Root here is needs to be the same type (or a parent type) of the node that you put in the child
///   scene as the root. For instance Spatial is used for this example.
fn instance_scene<Root>(scene: &PackedScene) -> Result<Ref<Root, Unique>, ManageErrs>
where
    Root: gdnative::object::GodotObject<Memory = ManuallyManaged> + SubClass<Node>,
{
    let instance = scene
        .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
        .ok_or(ManageErrs::CouldNotMakeInstance)?;
    let instance = unsafe { instance.assume_unique() };

    instance
        .try_cast::<Root>()
        .map_err(|instance| ManageErrs::RootClassNotSpatial(instance.name().to_string()))
}

fn update_panel(owner: &Spatial, num_children: i64) {
    // Here is how we call into the panel. First we get its node (we might have saved it
    //   from earlier)
    let panel_node_opt = owner.get_parent().and_then(|parent| {
        let parent = unsafe { parent.assume_safe() };
        parent.find_node("Panel", true, false)
    });

    if let Some(panel_node) = panel_node_opt {
        let panel_node = unsafe { panel_node.assume_safe() };

        // Put the Node
        let mut as_variant = Variant::new(panel_node);
        let result =
            unsafe { as_variant.call("set_num_children", &[Variant::new(num_children as u64)]) };

        match result {
            Ok(_) => godot_print!("Called Panel OK."),
            Err(_) => godot_print!("Error calling Panel"),
        }
    } else {
        godot_print!("Could not find panel node");
    }
}

struct SceneCreateLibrary;

#[gdnative::init::callbacks]
impl GDNativeCallbacks for SceneCreateLibrary {
    fn nativescript_init(handle: InitHandle) {
        handle.add_class::<SceneCreate>();
    }
}
