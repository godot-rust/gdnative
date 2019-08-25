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
            name: "something_happens",
            args: &[],
        });

        builder.add_signal(init::Signal {
            name: "something_happens_with_data",
            // optionally user can specify predefined list of signal arguments using init::SignalArgument
            args: &[],
        });
    }

    fn _init(_owner: gdnative::Node) -> Self {
        SignalEmitter {
            timer: 0.0,
            data: 100,
        }
    }

    #[export]
    fn _ready(&mut self, _owner: Node) {}

    #[export]
    fn _process(&mut self, mut owner: Node, delta: f64) {
        if self.timer < 1.0 {
            self.timer += delta;
            return;
        }
        self.timer = 0.0;
        self.data += 1;
        unsafe {
            if self.data % 2 == 0 {
                owner.emit_signal(GodotString::from_str("something_happens"), &[]);
            } else {
                owner.emit_signal(
                    GodotString::from_str("something_happens_with_data"),
                    &[Variant::from_i64(self.data)],
                );
            }
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
    fn _init(_owner: gdnative::Label) -> Self {
        SignalSubscriber { times_received: 0 }
    }

    #[export]
    unsafe fn _ready(&mut self, owner: Label) {
        let emitter = &mut owner
            .get_node(NodePath::from_str("../SignalEmitter"))
            .unwrap();
        let object = &owner.to_object();
        emitter
            .connect(
                GodotString::from_str("something_happens"),
                Some(*object),
                GodotString::from_str("notify"),
                VariantArray::new(),
                0,
            )
            .unwrap();
        emitter
            .connect(
                GodotString::from_str("something_happens_with_data"),
                Some(*object),
                GodotString::from_str("notify_with_data"),
                VariantArray::new(),
                0,
            )
            .unwrap();
    }

    #[export]
    fn notify(&mut self, mut _owner: Label) {
        self.times_received += 1;
        let msg = format!("Received signal {} times", self.times_received);

        unsafe {
            _owner.set_text(GodotString::from_str(msg.as_str()));
        }
    }

    #[export]
    fn notify_with_data(&mut self, mut _owner: Label, data: Variant) {
        let msg = format!("Received data {}", data.try_to_u64().unwrap());

        unsafe {
            _owner.set_text(GodotString::from_str(msg.as_str()));
        }
    }
}

fn init(handle: init::InitHandle) {
    handle.add_class::<SignalEmitter>();
    handle.add_class::<SignalSubscriber>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
