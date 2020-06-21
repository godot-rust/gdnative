use gdnative::api::*;
use gdnative::*;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_owner_free_ub();

    status
}

pub(crate) fn register(handle: init::InitHandle) {
    handle.add_class::<Bar>();
}

struct Bar(i64, Option<Arc<AtomicUsize>>);

impl NativeClass for Bar {
    type Base = Node;
    type UserData = user_data::RwLockData<Bar>;
    fn class_name() -> &'static str {
        "Bar"
    }
    fn init(_owner: &Node) -> Bar {
        Bar(42, None)
    }
    fn register_properties(_builder: &init::ClassBuilder<Self>) {}
}

impl Bar {
    fn set_drop_counter(&mut self, counter: Arc<AtomicUsize>) {
        self.1 = Some(counter);
    }
}

#[methods]
impl Bar {
    #[export]
    fn free_is_not_ub(&mut self, owner: &Node) -> bool {
        unsafe {
            owner.claim().free();
        }
        assert_eq!(42, self.0, "self should not point to garbage");
        true
    }

    #[export]
    fn set_script_is_not_ub(&mut self, owner: &Node) -> bool {
        owner.set_script(None);
        assert_eq!(42, self.0, "self should not point to garbage");
        true
    }
}

impl Drop for Bar {
    fn drop(&mut self) {
        let counter = self.1.take().expect("drop counter should be set");
        counter.fetch_add(1, AtomicOrdering::AcqRel);
        self.0 = 0;
    }
}

fn test_owner_free_ub() -> bool {
    println!(" -- test_owner_free_ub");

    let ok = std::panic::catch_unwind(|| {
        let drop_counter = Arc::new(AtomicUsize::new(0));

        {
            let bar = Instance::<Bar>::new();
            let bar = unsafe { bar.assume_safe() };

            bar.map_mut(|bar, _| bar.set_drop_counter(drop_counter.clone()))
                .expect("lock should not fail");

            assert_eq!(
                Some(true),
                bar.base()
                    .call("set_script_is_not_ub".into(), &[])
                    .try_to_bool()
            );

            unsafe {
                bar.claim().free();
            }
        }

        {
            let bar = Instance::<Bar>::new();
            let bar = unsafe { bar.assume_safe() };
            bar.map_mut(|bar, _| bar.set_drop_counter(drop_counter.clone()))
                .expect("lock should not fail");

            assert_eq!(
                Some(true),
                bar.base().call("free_is_not_ub".into(), &[]).try_to_bool()
            );
        }

        // the values are eventually dropped
        assert_eq!(2, drop_counter.load(AtomicOrdering::Acquire));
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_owner_free_ub failed");
    }

    ok
}
