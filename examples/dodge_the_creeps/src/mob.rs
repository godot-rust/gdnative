use gdnative::*;

#[derive(NativeClass)]
#[inherit(RigidBody2D)]
#[user_data(user_data::MutexData<Mob>)]
pub struct Mob {
    #[property(default = 150.0)]
    min_speed: f32,
    #[property(default = 250.0)]
    max_speed: f32,

    animation: MobType,
}

#[derive(Copy, Clone)]
enum MobType {
    Walk,
    Swim,
    Fly,
}

impl MobType {
    fn to_str(t: MobType) -> String {
        match t {
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
        self.animation = MOB_TYPES[1];
    }

    #[export]
    unsafe fn _on_Visibility_screen_exited(&self, mut owner: RigidBody2D) {
        owner.queue_free()
    }
}
