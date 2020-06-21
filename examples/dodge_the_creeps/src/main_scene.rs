use crate::extensions::NodeExt as _;
use crate::hud;
use crate::mob;
use crate::player;
use gdnative::api::*;
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
#[user_data(user_data::LocalCellData<Main>)]
pub struct Main {
    #[property]
    mob: Ref<PackedScene>,
    score: i64,
}

#[methods]
impl Main {
    fn _init(_owner: &Node) -> Self {
        Main {
            mob: PackedScene::new(),
            score: 0,
        }
    }

    #[export]
    fn game_over(&self, owner: &Node) {
        let score_timer: &Timer = unsafe { owner.get_typed_node("score_timer") };
        let mob_timer: &Timer = unsafe { owner.get_typed_node("mob_timer") };

        score_timer.stop();
        mob_timer.stop();

        let hud_node: &CanvasLayer = unsafe { owner.get_typed_node("hud") };
        hud_node
            .cast_instance::<hud::HUD>()
            .and_then(|hud| hud.map(|x, o| x.show_game_over(o)).ok())
            .unwrap_or_else(|| godot_print!("Unable to get hud"));
    }

    #[export]
    fn new_game(&mut self, owner: &Node) {
        let start_position: &Position2D = unsafe { owner.get_typed_node("start_position") };
        let player: &Area2D = unsafe { owner.get_typed_node("player") };
        let start_timer: &Timer = unsafe { owner.get_typed_node("start_timer") };

        self.score = 0;

        player
            .cast_instance::<player::Player>()
            .and_then(|player| {
                player
                    .map(|x, o| x.start(o, start_position.position()))
                    .ok()
            })
            .unwrap_or_else(|| godot_print!("Unable to get player"));

        start_timer.start(0.0);

        let hud_node: &CanvasLayer = unsafe { owner.get_typed_node("hud") };
        hud_node
            .cast_instance::<hud::HUD>()
            .and_then(|hud| {
                hud.map(|x, o| {
                    x.update_score(o, self.score);
                    x.show_message(o, "Get Ready".into());
                })
                .ok()
            })
            .unwrap_or_else(|| godot_print!("Unable to get hud"));
    }

    #[export]
    fn on_start_timer_timeout(&self, owner: &Node) {
        let mob_timer: &Timer = unsafe { owner.get_typed_node("mob_timer") };
        let score_timer: &Timer = unsafe { owner.get_typed_node("score_timer") };
        mob_timer.start(0.0);
        score_timer.start(0.0);
    }

    #[export]
    fn on_score_timer_timeout(&mut self, owner: &Node) {
        self.score += 1;

        let hud_node: &CanvasLayer = unsafe { owner.get_typed_node("hud") };
        hud_node
            .cast_instance::<hud::HUD>()
            .and_then(|hud| hud.map(|x, o| x.update_score(o, self.score)).ok())
            .unwrap_or_else(|| godot_print!("Unable to get hud"));
    }

    #[export]
    fn on_mob_timer_timeout(&self, owner: &Node) {
        let mob_spawn_location: &PathFollow2D =
            unsafe { owner.get_typed_node("mob_path/mob_spawn_locations") };

        let mob_scene: &RigidBody2D = instance_scene(&self.mob).unwrap();

        let mut rng = rand::thread_rng();
        let offset = rng.gen_range(std::u32::MIN, std::u32::MAX);

        mob_spawn_location.set_offset(offset.into());
        owner.add_child(Some(mob_scene.to_node()), false);

        let mut direction = mob_spawn_location.rotation() + PI / 2.0;

        mob_scene.set_position(mob_spawn_location.position());

        direction += rng.gen_range(-PI / 4.0, PI / 4.0);
        mob_scene.set_rotation(direction);
        let d = direction as f32;

        let mob = mob_scene.cast_instance::<mob::Mob>().unwrap();

        mob.map(|x, mob_owner| {
            mob_scene
                .set_linear_velocity(Vector2::new(rng.gen_range(x.min_speed, x.max_speed), 0.0));

            mob_scene
                .set_linear_velocity(mob_scene.linear_velocity().rotated(Angle { radians: d }));

            let hud_node: &CanvasLayer = unsafe { owner.get_typed_node("hud") };
            let hud = hud_node.cast_instance::<hud::HUD>().unwrap();

            hud.map(|_, o| {
                o.connect(
                    "start_game".into(),
                    Some(mob_owner.to_object()),
                    "on_start_game".into(),
                    VariantArray::new_shared(),
                    0,
                )
                .unwrap();
            })
            .unwrap();
        })
        .unwrap();
    }
}

/// Root here is needs to be the same type (or a parent type) of the node that you put in the child
///   scene as the root. For instance Spatial is used for this example.
fn instance_scene<Root>(scene: &PackedScene) -> Result<&Root, ManageErrs>
where
    Root: gdnative::GodotObject,
{
    let inst_option = scene.instance(PackedScene::GEN_EDIT_STATE_DISABLED);

    if let Some(instance) = inst_option {
        let instance = unsafe { instance.assume_safe() };
        if let Some(instance_root) = instance.cast::<Root>() {
            Ok(instance_root)
        } else {
            Err(ManageErrs::RootClassNotRigidBody2D(
                instance.name().to_string(),
            ))
        }
    } else {
        Err(ManageErrs::CouldNotMakeInstance)
    }
}
