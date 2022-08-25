use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(CanvasLayer)]
#[user_data(user_data::ArcData<Hud>)]
#[register_with(Self::register_hud)]
pub struct Hud;

#[methods]
impl Hud {
    fn register_hud(builder: &ClassBuilder<Self>) {
        builder.signal("start_game").done();
    }

    fn new(_owner: &CanvasLayer) -> Self {
        Hud
    }

    #[method]
    pub fn show_message(&self, #[base] owner: &CanvasLayer, text: String) {
        let message_label = unsafe { owner.get_node_as::<Label>("message_label").unwrap() };
        message_label.set_text(text);
        message_label.show();

        let timer = unsafe { owner.get_node_as::<Timer>("message_timer").unwrap() };
        timer.start(0.0);
    }

    pub fn show_game_over(&self, owner: &CanvasLayer) {
        self.show_message(owner, "Game Over".into());

        let message_label = unsafe { owner.get_node_as::<Label>("message_label").unwrap() };
        message_label.set_text("Dodge the\nCreeps!");
        message_label.show();

        let button = unsafe { owner.get_node_as::<Button>("start_button").unwrap() };
        button.show();
    }

    #[method]
    pub fn update_score(&self, #[base] owner: &CanvasLayer, score: i64) {
        let label = unsafe { owner.get_node_as::<Label>("score_label").unwrap() };
        label.set_text(score.to_string());
    }

    #[method]
    fn on_start_button_pressed(&self, #[base] owner: &CanvasLayer) {
        let button = unsafe { owner.get_node_as::<Button>("start_button").unwrap() };
        button.hide();
        owner.emit_signal("start_game", &[]);
    }

    #[method]
    fn on_message_timer_timeout(&self, #[base] owner: &CanvasLayer) {
        let message_label = unsafe { owner.get_node_as::<Label>("message_label").unwrap() };
        message_label.hide()
    }
}
