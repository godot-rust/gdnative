use gdnative::prelude::*;

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
    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.signal("tick").done();

        builder
            .signal("tick_with_data")
            // Argument list used by the editor for GUI and generation of GDScript handlers. It can be omitted if the signal is only used from code.
            .with_param_default("data", Variant::new(100))
            .done();
    }

    fn new(_owner: &Node) -> Self {
        SignalEmitter {
            timer: 0.0,
            data: 100,
        }
    }

    #[method]
    fn _process(&mut self, #[base] owner: &Node, delta: f64) {
        if self.timer < 1.0 {
            self.timer += delta;
            return;
        }
        self.timer = 0.0;
        self.data += 1;

        if self.data % 2 == 0 {
            owner.emit_signal("tick", &[]);
        } else {
            owner.emit_signal("tick_with_data", &[Variant::new(self.data)]);
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
    fn new(_owner: &Label) -> Self {
        SignalSubscriber { times_received: 0 }
    }

    #[method]
    fn _ready(&mut self, #[base] owner: TRef<Label>) {
        let emitter = &mut owner.get_node("../SignalEmitter").unwrap();
        let emitter = unsafe { emitter.assume_safe() };

        emitter
            .connect("tick", owner, "notify", VariantArray::new_shared(), 0)
            .unwrap();
        emitter
            .connect(
                "tick_with_data",
                owner,
                "notify_with_data",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();
    }

    #[method]
    fn notify(&mut self, #[base] owner: &Label) {
        self.times_received += 1;
        let msg = format!("Received signal \"tick\" {} times", self.times_received);

        owner.set_text(msg);
    }

    #[method]
    fn notify_with_data(&mut self, #[base] owner: &Label, data: Variant) {
        let msg = format!(
            "Received signal \"tick_with_data\" with data {}",
            data.try_to::<u64>().unwrap()
        );

        owner.set_text(msg);
    }
}

struct SignalLibrary;

#[gdnative::init::callbacks]
impl GDNativeCallbacks for SignalLibrary {
    fn nativescript_init(handle: InitHandle) {
        handle.add_class::<SignalEmitter>();
        handle.add_class::<SignalSubscriber>();
    }
}
