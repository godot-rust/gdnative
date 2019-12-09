use specs::prelude::*;
use sync::{register_gd_components, Pos};

pub struct YourState {
    /// This is a useful global container where you can put your junk is later going to be protected
    /// By a mutex
    world: World, // Add your stuff here if you like...
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ExampleComponent {
    pub child_id: u32,
}

impl Component for ExampleComponent {
    type Storage = VecStorage<Self>;
}

impl YourState {
    pub fn new() -> Self {
        // For more on how to set up a Specs world, be sure to see the specs documentation on
        //  docs.rs
        let mut world = World::new();

        // In my project (The Recall Singularity - https://twitter.com/RecallSingular1),
        //   I register some components in a different crate which provides the game logic.
        //   Then I add Godot tracking components to the world later using some hooks. So remember
        //   this stuff is dynamic (yay!)

        world.register::<Pos>();
        world.register::<ExampleComponent>();

        register_gd_components(&mut world);

        YourState { world }
    }

    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}
