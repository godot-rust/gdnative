use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
#[user_data(user_data::MutexData<Main>)]
#[register_with(register_main)]
// TODO: Store child nodes in the struct.
pub struct Main {
    mob: PackedScene,
    score: i64,
}

unsafe impl Send for Main {}

fn register_main(builder: &init::ClassBuilder<Main>) {
    builder.add_property(init::Property {
        name: "base/mob",
        default: load_scene("res://Mob.tscn"),
        hint: init::PropertyHint::None,
        getter: |this: &Main| this.mob.clone(),
        setter: |this: &mut Main, v| this.mob = v,
        usage: init::PropertyUsage::DEFAULT,
    });
}

#[methods]
impl Main {
    fn _init(_owner: Node) -> Self {
        Main {
            mob: PackedScene::new(),
            score: 0,
        }
    }

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

        let mut hud = owner
            .get_node("HUD".into())
            .expect("Missing HUD")
            .cast::<CanvasLayer>()
            .expect("Unable to cast to CanvasLayer");
        // hud.call("update_score".into(), &[Variant::from(self.score)]);
        // hud.call("show_message".into(), &[Variant::from("Get Ready")]);
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
            .get_node("StartTimer".into())
            .expect("Missing StartTimer")
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
        let mob_spawn_location = owner
            .get_node("MobPath/MobSpawnLocation".into())
            .expect("Missing MobPath/MobSpawnLocation")
            .cast::<PathFollow2D>()
            .expect("Unable to cast to PathFollow2D");

        let mob = self.mob.instance(0);
        owner.add_child(mob, false);

        // set the direction, rotation and velocity
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
