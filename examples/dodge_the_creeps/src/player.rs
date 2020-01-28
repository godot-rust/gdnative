use gdnative::*;

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
    fn register_player(builder: &init::ClassBuilder<Self>) {
        builder.add_signal(init::Signal {
            name: "hit",
            args: &[],
        });
    }

    fn _init(_owner: Area2D) -> Self {
        Player {
            speed: 400.0,
            screen_size: Vector2::new(0.0, 0.0),
        }
    }

    #[export]
    unsafe fn _ready(&mut self, mut owner: Area2D) {
        self.screen_size = owner.get_viewport().unwrap().get_size();
        owner.hide();

        let object = &owner.to_object();

        owner
            .connect(
                GodotString::from_str("hit"),
                Some(*object),
                GodotString::from_str("player_hit"),
                VariantArray::new(),
                0,
            )
            .unwrap();
    }

    #[export]
    unsafe fn _process(&mut self, mut owner: Area2D, delta: f32) {
        let input = Input::godot_singleton();
        let mut velocity = Vector2::new(0.0, 0.0);

        if Input::is_action_pressed(&input, GodotString::from_str("ui_right")) {
            velocity.x += 1.0
        }
        if Input::is_action_pressed(&input, GodotString::from_str("ui_left")) {
            velocity.x -= 1.0
        }
        if Input::is_action_pressed(&input, GodotString::from_str("ui_down")) {
            velocity.y += 1.0
        }
        if Input::is_action_pressed(&input, GodotString::from_str("ui_up")) {
            velocity.y -= 1.0
        }

        let mut animated_sprite = owner
            .get_node(NodePath::from_str("AnimatedSprite"))
            .expect("Missing AnimatedSprite")
            .cast::<AnimatedSprite>()
            .expect("Unable to cast to AnimatedSprite");

        if velocity.length() > 0.0 {
            velocity = velocity.normalize() * self.speed;

            let animation;

            if velocity.x != 0.0 {
                animation = "right";

                animated_sprite.set_flip_v(false);
                animated_sprite.set_flip_h(velocity.x < 0.0)
            } else {
                animation = "up";

                animated_sprite.set_flip_v(velocity.y > 0.0)
            }

            animated_sprite.play(GodotString::from_str(animation), false);
        } else {
            animated_sprite.stop();
        }

        let change = velocity * delta;
        let position =
            (owner.get_global_position() + change).clamp(Vector2::new(0.0, 0.0), self.screen_size);
        owner.set_global_position(position);
    }

    #[export]
    unsafe fn _on_Player_body_entered(&self, mut owner: Area2D, body: PhysicsBody2D) {
        owner.hide();
        owner.emit_signal("hit".into(), &[]);

        let mut collision_shape = owner
            .get_node("CollisionShape2D".into())
            .expect("Missing CollisionShape2D")
            .cast::<PathFollow2D>()
            .expect("Unable to cast to CollisionShape2D");

        collision_shape.set_deferred("disabled".into(), true.into());
        // $CollisionShape2D.set_deferred("disabled", true)
    }

    #[export]
    unsafe fn start(&self, mut owner: Area2D, pos: Vector2) {
        owner.set_global_position(pos);
        owner.show();
        // $CollisionShape2D.disabled = false
    }
}