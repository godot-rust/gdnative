use crate::extensions::NodeExt as _;
use crate::hud;
use crate::mob;
use crate::player;
use gdnative::api::*;
use gdnative::ref_kind::ManuallyManaged;
use gdnative::thread_access::{Shared, Unique};
use gdnative::*;
use rand::*;
use std::f64::consts::PI;

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
            mob: PackedScene::new().into_shared(),
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

        let mob_scene: Ref<RigidBody2D, _> = instance_scene(&self.mob);

        let mut rng = rand::thread_rng();
        let offset = rng.gen_range(std::u32::MIN, std::u32::MAX);

        mob_spawn_location.set_offset(offset.into());

        let mut direction = mob_spawn_location.rotation() + PI / 2.0;

        mob_scene.set_position(mob_spawn_location.position());

        direction += rng.gen_range(-PI / 4.0, PI / 4.0);
        mob_scene.set_rotation(direction);
        let d = direction as f32;

        let mob = mob_scene.cast_instance::<mob::Mob>().unwrap();

        mob.map(|x, mob_owner| {
            mob_owner
                .set_linear_velocity(Vector2::new(rng.gen_range(x.min_speed, x.max_speed), 0.0));

            mob_owner
                .set_linear_velocity(mob_owner.linear_velocity().rotated(Angle { radians: d }));

            let hud_node: &CanvasLayer = unsafe { owner.get_typed_node("hud") };
            let hud = hud_node.cast_instance::<hud::HUD>().unwrap();

            hud.map(|_, o| {
                o.connect(
                    "start_game".into(),
                    Some(unsafe { mob_owner.to_object().assume_shared() }),
                    "on_start_game".into(),
                    VariantArray::new_shared(),
                    0,
                )
                .unwrap();
            })
            .unwrap();
        })
        .unwrap();

        owner.add_child(
            Some(mob.into_base().cast::<Node>().unwrap().into_shared()),
            false,
        );
    }
}

/// Root here is needs to be the same type (or a parent type) of the node that you put in the child
///   scene as the root. For instance Spatial is used for this example.
fn instance_scene<Root>(scene: &Ref<PackedScene, Shared>) -> Ref<Root, Unique>
where
    Root: gdnative::GodotObject<RefKind = ManuallyManaged>,
{
    let scene = unsafe { scene.assume_safe() };

    let instance = scene
        .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
        .expect("should be able to instance scene");

    let instance = unsafe { instance.assume_unique() };

    instance
        .try_cast::<Root>()
        .expect("root node type should be correct")
}
