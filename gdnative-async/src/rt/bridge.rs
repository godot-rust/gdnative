use std::collections::HashMap;

use once_cell::sync::OnceCell;
use parking_lot::Mutex;

use gdnative_bindings::{Object, Reference};
use gdnative_core::core_types::{GodotError, Variant, VariantArray};
use gdnative_core::godot_site;
use gdnative_core::nativescript::export::method::{Method, Varargs};
use gdnative_core::nativescript::export::ClassBuilder;
use gdnative_core::nativescript::user_data::{ArcData, Map};
use gdnative_core::nativescript::{Instance, NativeClass, NativeClassMethods, RefInstance};
use gdnative_core::object::{ownership::Shared, TRef};

use crate::future::Resume;

// We need to keep our observers alive since `Object::connect` won't
static BRIDGES: OnceCell<Mutex<Pool>> = OnceCell::new();

pub(super) fn terminate() {
    if let Some(pool) = BRIDGES.get() {
        let mut pool = pool.lock();
        std::mem::take(&mut *pool);
    }
}

#[derive(Default)]
struct Pool {
    busy: HashMap<i64, Entry>,
    free: Vec<(i64, Instance<SignalBridge, Shared>)>,
    next_id: i64,
}

impl Pool {
    fn next_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

struct Entry {
    resume: Resume<Vec<Variant>>,

    // Just need to keep this alive.
    #[allow(dead_code)]
    obj: Instance<SignalBridge, Shared>,
}

pub(super) struct SignalBridge {
    id: i64,
}

impl NativeClass for SignalBridge {
    type Base = Reference;
    type UserData = ArcData<SignalBridge>;

    fn class_name() -> &'static str {
        // Sort of just praying that there will be no duplicates of this.
        "__GDNATIVE_ASYNC_INTERNAL__SignalBridge"
    }

    fn register_properties(_builder: &ClassBuilder<Self>) {}
}

impl SignalBridge {
    pub(crate) fn connect(
        source: TRef<Object>,
        signal: &str,
        resume: Resume<Vec<Variant>>,
    ) -> Result<(), GodotError> {
        assert!(
            super::REGISTRATION.get().is_some(),
            "async API must be registered before any async methods can be called"
        );

        let mut pool = BRIDGES.get_or_init(Mutex::default).lock();
        let (id, bridge) = pool.free.pop().unwrap_or_else(|| {
            let id = pool.next_id();
            let bridge = Instance::emplace(SignalBridge { id }).into_shared();
            (id, bridge)
        });

        source.connect(
            signal,
            bridge.base(),
            "_on_signal",
            VariantArray::new_shared(),
            Object::CONNECT_ONESHOT,
        )?;

        let entry = Entry {
            obj: bridge,
            resume,
        };

        assert!(pool.busy.insert(id, entry).is_none());

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct OnSignalFn;

impl Method<SignalBridge> for OnSignalFn {
    fn call(&self, this: RefInstance<'_, SignalBridge, Shared>, args: Varargs<'_>) -> Variant {
        let args = args.cloned().collect();

        let this_persist = this.clone().claim();

        this.script()
            .map(|s| {
                let (resume, args) = {
                    let mut pool = BRIDGES.get().unwrap().lock();
                    match pool.busy.remove(&s.id) {
                        Some(entry) => {
                            pool.free.push((s.id, this_persist));
                            (entry.resume, args)
                        }
                        None => {
                            gdnative_core::log::warn(
                                Self::site().unwrap(),
                                "`_on_signal` should only be called once per bridge object",
                            );
                            return;
                        }
                    }
                };

                resume.resume(args);
            })
            .unwrap();

        Variant::new()
    }

    fn site() -> Option<gdnative_core::log::Site<'static>> {
        Some(godot_site!(SignalBridge::_on_signal))
    }
}

impl NativeClassMethods for SignalBridge {
    fn register(builder: &ClassBuilder<Self>) {
        builder
            .build_method("_on_signal", OnSignalFn)
            .done_stateless();
    }
}
