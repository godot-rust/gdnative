mod entity_create;
mod entity_delete;
mod entity_move;
mod godot_sync_state;

use godot::*;
use specs::prelude::*;

pub use self::entity_create::GdEntityCreation;
pub use self::entity_delete::GDDeleteOldSpatials;
pub use self::entity_move::GdEntityTransform;
pub use self::godot_sync_state::GodotStateSync;
use nalgebra::Isometry3;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ExampleScene {
    ExampleCube,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Template3dName {
    // Create a new variant in this enum for every different thing that is in your game.

    //  Module(ModuleDrawType), -- In my game I have a list of modules that you can build
    //  SpaceObject(SpaceSubScenes), -- In my game I have stuff related to spaceships
    ExampleScenes(ExampleScene),
}

// Where is something in space, really?
pub struct Pos(pub Isometry3<f32>);

impl Component for Pos {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

/// GdSceneLoc is how you specify what sort of Godot Spatial you want to create, what to call it etc.
#[derive(Default, Debug, Clone)]
pub struct GdSceneLoc {
    /// We put this node underneath another node called parent_name (to collect similar things together)
    pub parent_name: &'static str,
    /// Name the new node that you are creating
    pub child_name: String,
    /// Specify which template you want to use (you'll need to load the relevant scene)
    pub template: Option<Template3dName>,
}

/// GdSpatial is how we track things that have been made in the past.
pub struct GdSpatial {
    /// Track the template used in Godot so we notice If you change which template you are using
    ///   in GdSceneLoc::template
    pub curr_template: Option<Template3dName>,
}

impl Component for GdSceneLoc {
    type Storage = VecStorage<Self>;
}

impl Component for GdSpatial {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

#[derive(Debug, Clone)]
pub enum Template3dEntity {
    None,
    Scene(PackedScene),
    Mesh(Mesh),
}

impl Default for Template3dEntity {
    fn default() -> Self {
        Template3dEntity::None
    }
}

pub fn register_gd_components(world: &mut specs::World) {
    println!("Registering components for world");
    world.register::<GdSpatial>();
    world.register::<GdSceneLoc>();
}
