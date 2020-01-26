use crate::hud;
use crate::mob;
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

// fn register_main(builder: &init::ClassBuilder<Main>) {
//     builder.add_property(init::Property {
//         name: "base/mob",
//         default: load_scene("res://Player.tscn"),
//         hint: init::PropertyHint::None,
//         getter: |this: &Main| this.mob.clone(),
//         setter: |this: &mut Main, v| this.mob = v,
//         usage: init::PropertyUsage::DEFAULT,
//     });
// }

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
            .get_node(NodePath::from_str("ScoreTimer"))
            .expect("Missing ScoreTimer")
            .cast::<Timer>()
            .expect("Unable to cast to Timer");

        let mut mob_timer = owner
            .get_node(NodePath::from_str("MobTimer"))
            .expect("Missing MobTimer")
            .cast::<Timer>()
            .expect("Unable to cast to Timer");

        score_timer.stop();
        mob_timer.stop();

        owner
            .get_node("HUD".into())
            .expect("Missing HUD")
            .cast::<CanvasLayer>()
            .expect("Unable to cast to CanvasLayer")
            .call("show_game_over".into(), &[]);
    }

    #[export]
    unsafe fn new_game(&mut self, owner: Node) {
        let mut start_position = owner
            .get_node(NodePath::from_str("StartPosition"))
            .expect("Missing StartPosition")
            .cast::<Position2D>()
            .expect("Unable to cast to Position2D");
        let mut player = owner
            .get_node(NodePath::from_str("Player"))
            .expect("Missing Player")
            .cast::<Area2D>()
            .expect("Unable to cast to Area2D");
        let mut start_timer = owner
            .get_node(NodePath::from_str("StartTimer"))
            .expect("Missing StartTimer")
            .cast::<Timer>()
            .expect("Unable to cast to Timer");

        self.score = 0;

        player.call(
            GodotString::from_str("start"),
            &[Variant::from(&start_position.get_position())],
        );

        start_timer.start(0.0);

        let mut hud_node = owner
            .get_node("HUD".into())
            .expect("Missing HUD")
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
    unsafe fn _on_StartTimer_timeout(&self, owner: Node) {
        owner
            .get_node("MobTimer".into())
            .expect("Missing MobTimer")
            .cast::<Timer>()
            .expect("Unable to cast to Timer")
            .start(0.0);
        owner
            .get_node("ScoreTimer".into())
            .expect("Missing ScoreTimer")
            .cast::<Timer>()
            .expect("Unable to cast to Timer")
            .start(0.0);
    }

    #[export]
    unsafe fn _on_ScoreTimer_timeout(&mut self, owner: Node) {
        self.score += 1;

        owner
            .get_node("HUD".into())
            .expect("Missing HUD")
            .cast::<CanvasLayer>()
            .expect("Unable to cast to CanvasLayer")
            .call("update_score".into(), &[Variant::from(self.score)]);
    }

    #[export]
    unsafe fn _on_MobTimer_timeout(&self, mut owner: Node) {
        let mut mob_spawn_location = owner
            .get_node("MobPath/MobSpawnLocations".into())
            .expect("Missing MobPath/MobSpawnLocations")
            .cast::<PathFollow2D>()
            .expect("Unable to cast to PathFollow2D");

        match instance_scene::<RigidBody2D>(&self.mob) {
            Ok(mut mob_scene) => {
                let mut rng = rand::thread_rng();
                mob_spawn_location.set_offset(rng.gen_range(0.0, std::f32::MAX).into());
                owner.add_child(Some(mob_scene.to_node()), false);

                let mut direction = mob_spawn_location.get_rotation() + PI / 2.0;

                mob_scene.set_position(mob_spawn_location.get_position());

                direction += rng.gen_range(-PI / 4.0, PI / 4.0);
                mob_scene.set_rotation(direction);

                match Instance::<mob::Mob>::try_from_unsafe_base(mob_scene) {
                    Some(mob) => {
                        let _ = mob.map(|x, _| {
                            mob_scene.set_linear_velocity(Vector2::new(
                                rng.gen_range(x.min_speed, x.max_speed),
                                0.0,
                            ));
                            let d = direction as f32;
                            mob_scene.set_linear_velocity(
                                mob_scene
                                    .get_linear_velocity()
                                    .rotated(Angle { radians: d }),
                            );
                        });
                    }
                    None => godot_print!("Unable to get mob"),
                }
            }
            Err(err) => godot_print!("Unable to instance mob: {:?}", err),
        }
    }
}

pub fn load_scene(path: &str) -> PackedScene {
    let scene = ResourceLoader::godot_singleton().load(
        GodotString::from_str(path), // could also use path.into() here
        GodotString::from_str("PackedScene"),
        false,
    );

    scene.and_then(|s| s.cast::<PackedScene>()).unwrap()
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
