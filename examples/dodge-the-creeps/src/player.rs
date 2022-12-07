use gdnative::api::{AnimatedSprite, Area2D, CollisionShape2D, PhysicsBody2D};
use gdnative::prelude::*;

/// The player "class"
#[derive(NativeClass)]
#[inherit(Area2D)]
#[user_data(user_data::MutexData<Player>)]
#[register_with(Self::register_player)]
pub struct Player {
    #[property(default = 400.0)]
    speed: f32,

    screen_size: Vector2,
}

#[methods]
impl Player {
    fn register_player(builder: &ClassBuilder<Self>) {
        builder.signal("hit").done()
    }

    fn new(_owner: &Area2D) -> Self {
        Player {
            speed: 400.0,
            screen_size: Vector2::new(0.0, 0.0),
        }
    }

    #[method]
    fn _ready(&mut self, #[base] owner: &Area2D) {
        let viewport = owner.get_viewport_rect();
        self.screen_size = viewport.size;
        owner.hide();
    }

    #[method]
    fn _process(&mut self, #[base] owner: &Area2D, delta: f32) {
        let animated_sprite = unsafe {
            owner
                .get_node_as::<AnimatedSprite>("animated_sprite")
                .unwrap()
        };

        let input = Input::godot_singleton();
        let mut velocity = Vector2::new(0.0, 0.0);

        // Note: exact=false by default, in Rust we have to provide it explicitly
        if Input::is_action_pressed(input, "ui_right", false) {
            velocity.x += 1.0
        }
        if Input::is_action_pressed(input, "ui_left", false) {
            velocity.x -= 1.0
        }
        if Input::is_action_pressed(input, "ui_down", false) {
            velocity.y += 1.0
        }
        if Input::is_action_pressed(input, "ui_up", false) {
            velocity.y -= 1.0
        }

        if velocity.length() > 0.0 {
            velocity = velocity.normalized() * self.speed;

            let animation;

            if velocity.x != 0.0 {
                animation = "right";

                animated_sprite.set_flip_v(false);
                animated_sprite.set_flip_h(velocity.x < 0.0)
            } else {
                animation = "up";

                animated_sprite.set_flip_v(velocity.y > 0.0)
            }

            animated_sprite.play(animation, false);
        } else {
            animated_sprite.stop();
        }

        let change = velocity * delta;
        let position = owner.global_position() + change;
        let position = Vector2::new(
            position.x.clamp(0.0, self.screen_size.x),
            position.y.clamp(0.0, self.screen_size.y),
        );
        owner.set_global_position(position);
    }

    #[method]
    fn on_player_body_entered(&self, #[base] owner: &Area2D, _body: Ref<PhysicsBody2D>) {
        owner.hide();
        owner.emit_signal("hit", &[]);

        let collision_shape = unsafe {
            owner
                .get_node_as::<CollisionShape2D>("collision_shape_2d")
                .unwrap()
        };

        collision_shape.set_deferred("disabled", true);
    }

    #[method]
    pub fn start(&self, #[base] owner: &Area2D, pos: Vector2) {
        owner.set_global_position(pos);
        owner.show();

        let collision_shape = unsafe {
            owner
                .get_node_as::<CollisionShape2D>("collision_shape_2d")
                .unwrap()
        };

        collision_shape.set_disabled(false);
    }
}
