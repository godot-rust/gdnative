use gdnative::{
    Button,
    Control,
    GodotObject,
    init::{ClassBuilder, Property, PropertyHint, PropertyUsage},
    Label,
    NativeClass,
    NodePath,
    TextEdit,
    ToVariant,
    user_data::MutexData,
    VariantArray,
};

use crate::events::Event;

pub(crate) struct Root {
    display: NodePath,
    incrementer: NodePath,
    button_text_entry: NodePath,
    deactivater: NodePath,

    connection_state: bool,
    count: u64,
}

impl Root {
    const DISPLAY_PATH: &'static str = "Display";
    const INCREMENTER_PATH: &'static str = "Incrementer";
    const BUTTON_TEXT_ENTRY_PATH: &'static str = "ButtonTextEntry";
    const DEACTIVATER_PATH: &'static str = "Deactivater";

    const INITIAL_CONNECTION_STATE: bool = false;
    const INITIAL_COUNT: u64 = 0;
}

impl Default for Root {
    fn default() -> Self {
        Self {
            display: Self::DISPLAY_PATH.into(),
            incrementer: Self::INCREMENTER_PATH.into(),
            button_text_entry: Self::BUTTON_TEXT_ENTRY_PATH.into(),
            deactivater: Self::DEACTIVATER_PATH.into(),

            connection_state: Self::INITIAL_CONNECTION_STATE,
            count: Self::INITIAL_COUNT,
        }
    }
}

impl NativeClass for Root {
    type Base = Control;
    type UserData = MutexData<Root>;

    fn class_name() -> &'static str {
        "Root"
    }

    fn init(_owner: Self::Base) -> Self {
        Default::default()
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
        builder.add_property(Property {
            name: "display",
            setter: |this: &mut Self, path| this.display = path,
            getter: |this: &Self| this.display.new_ref(),
            default: NodePath::from(Self::DISPLAY_PATH),
            hint: PropertyHint::NodePathToEditedNode,
            usage: PropertyUsage::DEFAULT,
        });
        builder.add_property(Property {
            name: "increment_button",
            setter: |this: &mut Self, path| this.incrementer = path,
            getter: |this: &Self| this.incrementer.new_ref(),
            default: NodePath::from(Self::INCREMENTER_PATH),
            hint: PropertyHint::NodePathToEditedNode,
            usage: PropertyUsage::DEFAULT,
        });
        builder.add_property(Property {
            name: "text_entry",
            setter: |this: &mut Self, path| this.button_text_entry = path,
            getter: |this: &Self| this.button_text_entry.new_ref(),
            default: NodePath::from(Self::BUTTON_TEXT_ENTRY_PATH),
            hint: PropertyHint::NodePathToEditedNode,
            usage: PropertyUsage::DEFAULT,
        });
        builder.add_property(Property {
            name: "disconnect_button",
            setter: |this: &mut Self, path| this.deactivater = path,
            getter: |this: &Self| this.deactivater.new_ref(),
            default: NodePath::from(Self::DEACTIVATER_PATH),
            hint: PropertyHint::NodePathToEditedNode,
            usage: PropertyUsage::DEFAULT,
        });

        builder.add_signal(Event::ConnectionStatus.signal_struct());
    }
}

impl Root {
    fn find_display(&self, owner: &Control) -> Option<Label> {
        Self::find_or_log(owner, &self.display, "display")
    }
    fn find_incrementer(&self, owner: &Control) -> Option<Button> {
        Self::find_or_log(owner, &self.incrementer, "incrementer")
    }
    fn find_button_text_entry(&self, owner: &Control) -> Option<TextEdit> {
        Self::find_or_log(owner, &self.button_text_entry, "button text entry")
    }
    fn find_deactivater(&self, owner: &Control) -> Option<Button> {
        Self::find_or_log(owner, &self.deactivater, "deactivater")
    }
    fn find_or_log<T: GodotObject>(owner: &Control, path: &NodePath, name: &str) -> Option<T> {
        let opt_node = unsafe { owner.get_node(path.new_ref()).and_then(|n| n.cast()) };
        if opt_node.is_none() {
            godot_warn!("Could not find {} at path {}!", name, path.to_string());
        }
        opt_node
    }
}

#[methods]
impl Root {
    #[export]
    fn _ready(&mut self, owner: Control) {
        // This following connection is set in the editor.
        if let Some(incrementer) = self.find_incrementer(&owner) {
            let res = Event::Pressed.connect(
                &incrementer,
                &owner,
                "increment",
            );
            match res {
                Ok(_) => (),
                Err(_) => (),
            }
        }

        if let Some(button_text_entry) = self.find_button_text_entry(&owner) {
            let mut binds = VariantArray::new();
            binds.push(&button_text_entry.to_variant());
            let res = Event::TextChanged.connect_with_binds(
                &button_text_entry,
                &owner,
                "change_incrementer_button_text",
                binds,
            );
            match res {
                Ok(_) => (),
                Err(_) => (),
            }
        }

        if let Some(deactivater) = self.find_deactivater(&owner) {
            let res = Event::Pressed.connect(
                &deactivater,
                &owner,
                "toggle_connection_state",
            );
            match res {
                Ok(_) => (),
                Err(_) => (),
            }
        }
    }

    #[export]
    fn increment(&mut self, owner: Control) {
        let mut display = self.find_display(&owner);
        self.count += 1;

        if let Some(display) = display.as_mut() {
            unsafe {
                display.set_text(format!("{}", self.count).into());
            }
        }
    }

    #[export]
    fn change_incrementer_button_text(&mut self, owner: Control, mut textbox: TextEdit) {
        let mut incrementer = self.find_incrementer(&owner);
        let text = unsafe { textbox.get_text() };

        if let Some(incrementer) = incrementer.as_mut() {
            unsafe {
                incrementer.set_text(text);
            }
        }
    }

    #[export]
    fn toggle_connection_state(&mut self, owner: Control) {
        let incrementer = self.find_incrementer(&owner);
        self.connection_state = !self.connection_state;

        if let Some(incrementer) = incrementer {
            if self.connection_state {
                let res = Event::Pressed.connect(incrementer, owner, "increment");
                match res {
                    Ok(_) => (),
                    Err(_) => (),
                }
                Event::emit_increment_button_connected_signal(owner);
            } else {
                Event::Pressed.disconnect(incrementer, owner, "increment");
                Event::emit_increment_button_disconnected_signal(owner);
            }
        }
    }
}
