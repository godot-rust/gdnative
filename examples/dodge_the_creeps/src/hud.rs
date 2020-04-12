use crate::extensions::NodeExt as _;
use gdnative::*;

#[derive(NativeClass)]
#[inherit(CanvasLayer)]
#[user_data(user_data::ArcData<HUD>)]
#[register_with(Self::register_hud)]
pub struct HUD;

#[methods]
impl HUD {
    fn register_hud(builder: &init::ClassBuilder<Self>) {
        builder.add_signal(init::Signal {
            name: "start_game",
            args: &[],
        });
    }

    fn _init(_owner: CanvasLayer) -> Self {
        HUD
    }

    #[export]
    pub unsafe fn show_message(&self, owner: CanvasLayer, text: String) {
        let mut message_label: Label = owner
            .get_typed_node("message_label")
            .expect("Cannot cast to Label");

        message_label.set_text(text.into());
        message_label.show();

        owner
            .get_typed_node::<Timer, _>("message_timer")
            .expect("Cannot cast to Timer")
            .start(0.0);
    }

    pub unsafe fn show_game_over(&self, owner: CanvasLayer) {
        self.show_message(owner, "Game Over".into());

        let mut message_label: Label = owner
            .get_typed_node("message_label")
            .expect("Cannot cast to Label");

        message_label.set_text("Dodge the\nCreeps!".into());
        message_label.show();

        owner
            .get_typed_node::<Button, _>("start_button")
            .expect("Cannot cast to Button")
            .show();
    }

    #[export]
    pub unsafe fn update_score(&self, owner: CanvasLayer, score: i64) {
        owner
            .get_typed_node::<Label, _>("score_label")
            .expect("Cannot cast to Label")
            .set_text(score.to_string().into());
    }

    #[export]
    unsafe fn on_start_button_pressed(&self, mut owner: CanvasLayer) {
        owner
            .get_typed_node::<Button, _>("start_button")
            .expect("Cannot cast to Button")
            .hide();

        owner.emit_signal("start_game".into(), &[]);
    }

    #[export]
    unsafe fn on_message_timer_timeout(&self, owner: CanvasLayer) {
        owner
            .get_typed_node::<Label, _>("message_label")
            .expect("Cannot cast to Label")
            .hide()
    }
}
