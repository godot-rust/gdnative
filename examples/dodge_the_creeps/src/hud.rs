use crate::extensions::NodeExt as _;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(CanvasLayer)]
#[user_data(user_data::ArcData<HUD>)]
#[register_with(Self::register_hud)]
pub struct HUD;

#[methods]
impl HUD {
    fn register_hud(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "start_game",
            args: &[],
        });
    }

    fn new(_owner: _) -> Self {
        HUD
    }

    #[export]
    pub fn show_message(&self, owner: _, text: String) {
        let message_label = unsafe { owner.get_typed_node::<Label, _>("message_label") };
        message_label.set_text(text);
        message_label.show();

        let timer = unsafe { owner.get_typed_node::<Timer, _>("message_timer") };
        timer.start(0.0);
    }

    pub fn show_game_over(&self, owner: TRef<CanvasLayer>) {
        self.show_message(owner, "Game Over".into());

        let message_label = unsafe { owner.get_typed_node::<Label, _>("message_label") };
        message_label.set_text("Dodge the\nCreeps!");
        message_label.show();

        let button = unsafe { owner.get_typed_node::<Button, _>("start_button") };
        button.show();
    }

    #[export]
    pub fn update_score(&self, owner: _, score: i64) {
        let label = unsafe { owner.get_typed_node::<Label, _>("score_label") };
        label.set_text(score.to_string());
    }

    #[export]
    fn on_start_button_pressed(&self, owner: _) {
        let button = unsafe { owner.get_typed_node::<Button, _>("start_button") };
        button.hide();
        owner.emit_signal("start_game", &[]);
    }

    #[export]
    fn on_message_timer_timeout(&self, owner: _) {
        let message_label = unsafe { owner.get_typed_node::<Label, _>("message_label") };
        message_label.hide()
    }
}
