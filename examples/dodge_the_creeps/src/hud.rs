use gdnative::*;

#[derive(NativeClass)]
#[inherit(CanvasLayer)]
#[user_data(user_data::ArcData<HUD>)]
#[register_with(Self::register_hud)]
pub struct HUD {}

#[methods]
impl HUD {
    fn register_hud(builder: &init::ClassBuilder<Self>) {
        builder.add_signal(init::Signal {
            name: "start_game",
            args: &[],
        });
    }

    fn _init(_owner: CanvasLayer) -> Self {
        HUD {}
    }

    #[export]
    unsafe fn _ready(&self, owner: CanvasLayer) {}

    #[export]
    pub unsafe fn show_message(&self, owner: CanvasLayer, text: String) {
        let mut message_label = owner
            .get_node("MessageLabel".into())
            .expect("Missing MessageLabel")
            .cast::<Label>()
            .expect("Cannot cast to Label");

        message_label.set_text(text.into());
        message_label.show();

        owner
            .get_node("MessageTimer".into())
            .expect("Missing MessageTimer")
            .cast::<Timer>()
            .expect("Cannot cast to Timer")
            .start(0.0)
    }

    #[export]
    unsafe fn show_game_over(&self, owner: CanvasLayer) {
        self.show_message(owner, "Game Over".into());
        // yield($MessageTimer, "timeout")

        let mut message_label = owner
            .get_node("MessageLabel".into())
            .expect("Missing MessageLabel")
            .cast::<Label>()
            .expect("Cannot cast to Label");

        message_label.set_text("Dodge the\nCreeps!".into());
        message_label.show();

        // yield(get_tree().create_timer(1), 'timeout')

        owner
            .get_node("StartButton".into())
            .expect("Missing StartButton")
            .cast::<Button>()
            .expect("Cannot cast to Button")
            .show();
    }

    #[export]
    pub unsafe fn update_score(&self, owner: CanvasLayer, score: i64) {
        owner
            .get_node("ScoreLabel".into())
            .expect("Missing ScoreLabel")
            .cast::<Label>()
            .expect("Cannot cast to Label")
            .set_text(score.to_string().into());
    }

    #[export]
    unsafe fn _on_StartButton_pressed(&self, mut owner: CanvasLayer) {
        godot_print!("Start Button Pressed!");

        owner
            .get_node("StartButton".into())
            .expect("Missing StartButton")
            .cast::<Button>()
            .expect("Cannot cast to Button")
            .hide();

        owner.emit_signal("start_game".into(), &[]);
    }

    #[export]
    unsafe fn _on_MessageTimer_timeout(&self, owner: CanvasLayer) {
        owner
            .get_node("MessageLabel".into())
            .expect("Missing MessageLabel")
            .cast::<Label>()
            .expect("Cannot cast to Label")
            .hide()
    }
}
