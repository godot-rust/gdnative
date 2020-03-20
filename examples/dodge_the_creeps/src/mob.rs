use gdnative::*;
use rand::seq::SliceRandom;
use rand::*;

#[derive(NativeClass)]
#[inherit(RigidBody2D)]
#[user_data(user_data::MutexData<Mob>)]
pub struct Mob {
    #[property(default = 150.0)]
    pub min_speed: f32,
    #[property(default = 250.0)]
    pub max_speed: f32,

    animation: MobType,
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
    fn _init(_owner: RigidBody2D) -> Self {
        Mob {
            min_speed: 150.0,
            max_speed: 250.0,
            animation: MobType::Walk,
        }
    }

    #[export]
    unsafe fn _ready(&mut self, owner: RigidBody2D) {
        let mut rng = thread_rng();

        let mut animated_sprite = owner
            .get_node("animated_sprite".into())
            .expect("Missing animated_sprite")
            .cast::<AnimatedSprite>()
            .expect("Unable to cast to animated_sprite");

        animated_sprite.set_animation(MOB_TYPES.choose(&mut rng).unwrap().to_str().into())
    }

    #[export]
    unsafe fn on_visibility_screen_exited(&self, mut owner: RigidBody2D) {
        owner.queue_free()
    }

    #[export]
    unsafe fn on_start_game(&self, mut owner: RigidBody2D) {
        owner.queue_free();
    }
}
