use crate::hud;
use crate::mob;
use crate::player;
use gdnative::*;
use rand::*;
use std::f64::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub enum ManageErrs {
    CouldNotMakeInstance,
    RootClassNotRigidBody2D(String),
}

#[derive(NativeClass)]
#[inherit(Node)]
#[user_data(user_data::MutexData<Main>)]
// #[register_with(register_main)]
// TODO: Store child nodes in the struct.
pub struct Main {
    #[property]
    mob: PackedScene,
    score: i64,
}

unsafe impl Send for Main {}

#[methods]
impl Main {
    fn _init(_owner: Node) -> Self {
        Main {
            mob: PackedScene::new(),
            score: 0,
        }
    }

    #[export]
    fn _ready(&self, _owner: Node) {}

    #[export]
    unsafe fn game_over(&self, owner: Node) {
        let mut score_timer = owner
            .get_node(NodePath::from_str("score_timer"))
            .expect("Missing score_timer")
            .cast::<Timer>()
            .expect("Unable to cast to Timer");

        let mut mob_timer = owner
            .get_node(NodePath::from_str("mob_timer"))
            .expect("Missing mob_timer")
            .cast::<Timer>()
            .expect("Unable to cast to Timer");

        score_timer.stop();
        mob_timer.stop();

        let hud_node = owner
            .get_node("hud".into())
            .expect("Missing hud")
            .cast::<CanvasLayer>()
            .expect("Unable to cast to CanvasLayer");

        match Instance::<hud::HUD>::try_from_unsafe_base(hud_node) {
            Some(hud) => {
                let _ = hud.map(|x, o| {
                    x.show_game_over(o);
                });
                ()
            }
            None => godot_print!("Unable to get hud"),
        }
    }

    #[export]
    unsafe fn new_game(&mut self, owner: Node) {
        let start_position = owner
            .get_node(NodePath::from_str("start_position"))
            .expect("Missing start_position")
            .cast::<Position2D>()
            .expect("Unable to cast to Position2D");
        let player = owner
            .get_node(NodePath::from_str("player"))
            .expect("Missing player")
            .cast::<Area2D>()
            .expect("Unable to cast to Area2D");
        let mut start_timer = owner
            .get_node(NodePath::from_str("start_timer"))
            .expect("Missing start_timer")
            .cast::<Timer>()
            .expect("Unable to cast to Timer");

        self.score = 0;

        // player.call(
        //     GodotString::from_str("start"),
        //     &[Variant::from(&start_position.get_position())],
        // );

        match Instance::<player::Player>::try_from_unsafe_base(player) {
            Some(player) => {
                let _ = player.map(|x, o| x.start(o, start_position.get_position()));
                ()
            }
            None => godot_print!("Unable to get hud"),
        }

        start_timer.start(0.0);

        let hud_node = owner
            .get_node("hud".into())
            .expect("Missing hud")
            .cast::<CanvasLayer>()
            .expect("Unable to cast to CanvasLayer");

        match Instance::<hud::HUD>::try_from_unsafe_base(hud_node) {
            Some(hud) => {
                let _ = hud.map(|x, o| {
                    x.update_score(o, self.score);
                    x.show_message(o, "Get Ready".to_string())
                });
                ()
            }
            None => godot_print!("Unable to get hud"),
        }
    }

    #[export]
    unsafe fn on_start_timer_timeout(&self, owner: Node) {
        owner
            .get_node("mob_timer".into())
            .expect("Missing mob_timer")
            .cast::<Timer>()
            .expect("Unable to cast to Timer")
            .start(0.0);
        owner
            .get_node("score_timer".into())
            .expect("Missing score_timer")
            .cast::<Timer>()
            .expect("Unable to cast to Timer")
            .start(0.0);
    }

    #[export]
    unsafe fn on_score_timer_timeout(&mut self, owner: Node) {
        self.score += 1;

        let hud_node = owner
            .get_node("hud".into())
            .expect("Missing hud")
            .cast::<CanvasLayer>()
            .expect("Unable to cast to CanvasLayer");
        // .call("update_score".into(), &[Variant::from(self.score)]);

        match Instance::<hud::HUD>::try_from_unsafe_base(hud_node) {
            Some(hud) => {
                let _ = hud.map(|x, o| x.update_score(o, self.score));
                ()
            }
            None => godot_print!("Unable to get hud"),
        }
    }

    #[export]
    unsafe fn on_mob_timer_timeout(&self, mut owner: Node) {
        let mut mob_spawn_location = owner
            .get_node("mob_path/mob_spawn_locations".into())
            .expect("Missing mob_path/mob_spawn_locations")
            .cast::<PathFollow2D>()
            .expect("Unable to cast to PathFollow2D");

        match instance_scene::<RigidBody2D>(&self.mob) {
            Ok(mut mob_scene) => {
                let mut rng = rand::thread_rng();
                let offset = rng.gen_range(std::u32::MIN, std::u32::MAX);

                mob_spawn_location.set_offset(offset.into());
                owner.add_child(Some(mob_scene.to_node()), false);

                let mut direction = mob_spawn_location.get_rotation() + PI / 2.0;

                mob_scene.set_position(mob_spawn_location.get_position());

                direction += rng.gen_range(-PI / 4.0, PI / 4.0);
                mob_scene.set_rotation(direction);
                let d = direction as f32;

                match Instance::<mob::Mob>::try_from_unsafe_base(mob_scene) {
                    Some(mob) => {
                        let _ = mob.map(|x, mob_owner| {
                            mob_scene.set_linear_velocity(Vector2::new(
                                rng.gen_range(x.min_speed, x.max_speed),
                                0.0,
                            ));

                            mob_scene.set_linear_velocity(
                                mob_scene
                                    .get_linear_velocity()
                                    .rotated(Angle { radians: d }),
                            );

                            let hud_node = owner
                                .get_node("hud".into())
                                .expect("Missing hud")
                                .cast::<CanvasLayer>()
                                .expect("Unable to cast to CanvasLayer");

                            match Instance::<hud::HUD>::try_from_unsafe_base(hud_node) {
                                Some(hud) => {
                                    let _ = hud.map(|_, mut o| {
                                        let _ = o.connect(
                                            "start_game".into(),
                                            Some(mob_owner.to_object()),
                                            "on_start_game".into(),
                                            VariantArray::new(),
                                            0,
                                        );
                                    });
                                    ()
                                }
                                None => godot_print!("Unable to get hud"),
                            }
                        });
                    }
                    None => godot_print!("Unable to get mob"),
                }
            }
            Err(err) => godot_print!("Unable to instance mob: {:?}", err),
        }
    }
}

/// Root here is needs to be the same type (or a parent type) of the node that you put in the child
///   scene as the root. For instance Spatial is used for this example.
unsafe fn instance_scene<Root>(scene: &PackedScene) -> Result<Root, ManageErrs>
where
    Root: gdnative::GodotObject,
{
    let inst_option = scene.instance(1); // 0 - GEN_EDIT_STATE_DISABLED

    if let Some(instance) = inst_option {
        if let Some(instance_root) = instance.cast::<Root>() {
            Ok(instance_root)
        } else {
            Err(ManageErrs::RootClassNotRigidBody2D(
                instance.get_name().to_string(),
            ))
        }
    } else {
        Err(ManageErrs::CouldNotMakeInstance)
    }
}
