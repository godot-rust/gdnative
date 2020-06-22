use gdnative::api::{Label, Node};
use gdnative::*;

#[derive(NativeClass)]
#[inherit(Node)]
// register_with attribute can be used to specify custom register function for node signals and properties
#[register_with(Self::register_signals)]
struct SignalEmitter {
    timer: f64,
    data: i64,
}

#[methods]
impl SignalEmitter {
    fn register_signals(builder: &init::ClassBuilder<Self>) {
        builder.add_signal(init::Signal {
            name: "tick",
            args: &[],
        });

        builder.add_signal(init::Signal {
            name: "tick_with_data",
            // Argument list used by the editor for GUI and generation of GDScript handlers. It can be omitted if the signal is only used from code.
            args: &[init::SignalArgument {
                name: "data",
                default: Variant::from_i64(100),
                export_info: init::ExportInfo::new(VariantType::I64),
                usage: init::PropertyUsage::DEFAULT,
            }],
        });
    }

    fn _init(_owner: &Node) -> Self {
        SignalEmitter {
            timer: 0.0,
            data: 100,
        }
    }

    #[export]
    fn _process(&mut self, owner: &Node, delta: f64) {
        if self.timer < 1.0 {
            self.timer += delta;
            return;
        }
        self.timer = 0.0;
        self.data += 1;

        if self.data % 2 == 0 {
            owner.emit_signal(GodotString::from_str("tick"), &[]);
        } else {
            owner.emit_signal(
                GodotString::from_str("tick_with_data"),
                &[Variant::from_i64(self.data)],
            );
        }
    }
}

#[derive(NativeClass)]
#[inherit(Label)]
struct SignalSubscriber {
    times_received: i32,
}

#[methods]
impl SignalSubscriber {
    fn _init(_owner: &Label) -> Self {
        SignalSubscriber { times_received: 0 }
    }

    #[export]
    fn _ready(&mut self, owner: &Label) {
        let emitter = &mut owner
            .get_node(NodePath::from_str("../SignalEmitter"))
            .unwrap();
        let emitter = unsafe { emitter.assume_safe() };

        let object = unsafe { owner.to_object().assume_shared() };
        emitter
            .connect(
                GodotString::from_str("tick"),
                Some(object),
                GodotString::from_str("notify"),
                VariantArray::new_shared(),
                0,
            )
            .unwrap();
        emitter
            .connect(
                GodotString::from_str("tick_with_data"),
                Some(object),
                GodotString::from_str("notify_with_data"),
                VariantArray::new_shared(),
                0,
            )
            .unwrap();
    }

    #[export]
    fn notify(&mut self, owner: &Label) {
        self.times_received += 1;
        let msg = format!("Received signal \"tick\" {} times", self.times_received);

        owner.set_text(GodotString::from_str(msg.as_str()));
    }

    #[export]
    fn notify_with_data(&mut self, owner: &Label, data: Variant) {
        let msg = format!(
            "Received signal \"tick_with_data\" with data {}",
            data.try_to_u64().unwrap()
        );

        owner.set_text(GodotString::from_str(msg.as_str()));
    }
}

fn init(handle: init::InitHandle) {
    handle.add_class::<SignalEmitter>();
    handle.add_class::<SignalSubscriber>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
