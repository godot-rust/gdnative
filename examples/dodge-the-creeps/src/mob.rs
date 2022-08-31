use gdnative::api::{AnimatedSprite, RigidBody2D};
use gdnative::prelude::*;
use rand::seq::SliceRandom;

#[derive(NativeClass)]
#[inherit(RigidBody2D)]
#[user_data(user_data::MutexData<Mob>)]
pub struct Mob {
    #[property(default = 150.0)]
    pub min_speed: f32,
    #[property(default = 250.0)]
    pub max_speed: f32,
}

#[derive(Copy, Clone)]
enum MobType {
    Walk,
    Swim,
    Fly,
}

impl MobType {
    fn to_str(self) -> String {
        match self {
            MobType::Walk => "walk".to_string(),
            MobType::Swim => "swim".to_string(),
            MobType::Fly => "fly".to_string(),
        }
    }
}

const MOB_TYPES: [MobType; 3] = [MobType::Walk, MobType::Swim, MobType::Fly];

#[methods]
impl Mob {
    fn new(_owner: &RigidBody2D) -> Self {
        Mob {
            min_speed: 150.0,
            max_speed: 250.0,
        }
    }

    #[method]
    fn _ready(&mut self, #[base] owner: &RigidBody2D) {
        let mut rng = rand::thread_rng();
        let animated_sprite = unsafe {
            owner
                .get_node_as::<AnimatedSprite>("animated_sprite")
                .unwrap()
        };
        animated_sprite.set_animation(MOB_TYPES.choose(&mut rng).unwrap().to_str())
    }

    #[method]
    fn on_visibility_screen_exited(&self, #[base] owner: &RigidBody2D) {
        unsafe {
            owner.assume_unique().queue_free();
        }
    }

    #[method]
    fn on_start_game(&self, #[base] owner: &RigidBody2D) {
        unsafe {
            owner.assume_unique().queue_free();
        }
    }
}
