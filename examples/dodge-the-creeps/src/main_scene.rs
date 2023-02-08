use crate::hud;
use crate::mob;
use crate::player;
use gdnative::api::{PathFollow2D, Position2D, RigidBody2D};
use gdnative::prelude::*;
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
    fn new(_owner: &Node) -> Self {
        Main {
            mob: PackedScene::new().into_shared(),
            score: 0,
        }
    }

    #[method]
    fn game_over(&self, #[base] owner: &Node) {
        let score_timer = unsafe { owner.get_node_as::<Timer>("score_timer").unwrap() };
        let mob_timer = unsafe { owner.get_node_as::<Timer>("mob_timer").unwrap() };

        score_timer.stop();
        mob_timer.stop();

        let hud = unsafe { owner.get_node_as_instance::<hud::Hud>("hud").unwrap() };
        hud.map(|x, o| x.show_game_over(&o))
            .ok()
            .unwrap_or_else(|| godot_print!("Unable to get hud"));
    }

    #[method]
    fn new_game(&mut self, #[base] owner: &Node) {
        let start_position = unsafe { owner.get_node_as::<Position2D>("start_position").unwrap() };
        let player = unsafe {
            owner
                .get_node_as_instance::<player::Player>("player")
                .unwrap()
        };
        let start_timer = unsafe { owner.get_node_as::<Timer>("start_timer").unwrap() };

        self.score = 0;

        player
            .map(|x, o| x.start(&o, start_position.position()))
            .ok()
            .unwrap_or_else(|| godot_print!("Unable to get player"));

        start_timer.start(0.0);

        let hud = unsafe { owner.get_node_as_instance::<hud::Hud>("hud").unwrap() };
        hud.map(|x, o| {
            x.update_score(&o, self.score);
            x.show_message(&o, "Get Ready".into());
        })
        .ok()
        .unwrap_or_else(|| godot_print!("Unable to get hud"));
    }

    #[method]
    fn on_start_timer_timeout(&self, #[base] owner: &Node) {
        let mob_timer = unsafe { owner.get_node_as::<Timer>("mob_timer").unwrap() };
        let score_timer = unsafe { owner.get_node_as::<Timer>("score_timer").unwrap() };
        mob_timer.start(0.0);
        score_timer.start(0.0);
    }

    #[method]
    fn on_score_timer_timeout(&mut self, #[base] owner: &Node) {
        self.score += 1;

        let hud = unsafe { owner.get_node_as_instance::<hud::Hud>("hud").unwrap() };
        hud.map(|x, o| x.update_score(&o, self.score))
            .ok()
            .unwrap_or_else(|| godot_print!("Unable to get hud"));
    }

    #[method]
    fn on_mob_timer_timeout(&self, #[base] owner: &Node) {
        let mob_spawn_location = unsafe {
            owner
                .get_node_as::<PathFollow2D>("mob_path/mob_spawn_locations")
                .unwrap()
        };

        let mob_scene: Ref<RigidBody2D, _> = instance_scene(&self.mob);

        let mut rng = rand::thread_rng();
        let offset = rng.gen_range(u32::MIN..u32::MAX);

        mob_spawn_location.set_offset(offset.into());

        let mut direction = mob_spawn_location.rotation() + PI / 2.0;

        mob_scene.set_position(mob_spawn_location.position());

        direction += rng.gen_range(-PI / 4.0..PI / 4.0);
        mob_scene.set_rotation(direction);
        let d = direction as f32;

        let mob_scene = unsafe { mob_scene.into_shared().assume_safe() };
        owner.add_child(mob_scene, false);

        let mob = mob_scene.cast_instance::<mob::Mob>().unwrap();

        mob.map(|x, mob_owner| {
            mob_owner
                .set_linear_velocity(Vector2::new(rng.gen_range(x.min_speed..x.max_speed), 0.0));

            mob_owner.set_linear_velocity(mob_owner.linear_velocity().rotated(d));

            let hud = unsafe { owner.get_node_as_instance::<hud::Hud>("hud").unwrap() };

            hud.map(|_, o| {
                o.connect(
                    "start_game",
                    mob_owner,
                    "on_start_game",
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
fn instance_scene<Root>(scene: &Ref<PackedScene, Shared>) -> Ref<Root, Unique>
where
    Root: gdnative::object::GodotObject<Memory = ManuallyManaged> + SubClass<Node>,
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
