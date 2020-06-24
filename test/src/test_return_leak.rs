use gdnative::api::*;
use gdnative::*;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_return_leak();

    status
}

pub(crate) fn register(handle: init::InitHandle) {
    handle.add_class::<Probe>();
}

/// Probe type to track dropping of the Reference it's attached to. As the leak
/// only happens with ptrcalls, this will have to inherit a Reference class that is
/// accepted somewhere in the Godot API as a property. Here, the chosen type is
/// `AnimationNodeAdd2`, of the property `AnimationTree::tree_root`.
struct Probe {
    drop_count: Option<Arc<AtomicUsize>>,
}

impl NativeClass for Probe {
    type Base = AnimationNodeAdd2;
    type UserData = user_data::RwLockData<Probe>;

    fn class_name() -> &'static str {
        "ReturnLeakProbe"
    }

    fn init(_owner: &AnimationNodeAdd2) -> Probe {
        Probe { drop_count: None }
    }

    fn register_properties(_builder: &init::ClassBuilder<Self>) {}
}

impl Probe {
    fn set_drop_counter(&mut self, counter: Arc<AtomicUsize>) {
        self.drop_count = Some(counter);
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        let counter = self.drop_count.take().expect("drop counter should be set");
        counter.fetch_add(1, AtomicOrdering::AcqRel);
    }
}

#[methods]
impl Probe {}

fn test_return_leak() -> bool {
    println!(" -- test_return_leak");

    let ok = std::panic::catch_unwind(|| {
        let drop_counter = Arc::new(AtomicUsize::new(0));

        // The object used for its ptrcall getter
        let animation_tree = AnimationTree::new();

        // Create an instance of the probe, and drop the reference after setting the property
        // to it. After this block, the only reference should be the one in `animation_tree`.
        {
            let probe = Probe::new_instance();
            probe
                .map_mut(|probe, _| probe.set_drop_counter(drop_counter.clone()))
                .expect("lock should not fail");

            let base = probe.into_base().into_shared();
            animation_tree.set_tree_root(base.cast().unwrap());
        }

        assert_eq!(0, drop_counter.load(AtomicOrdering::Acquire));

        // Take the reference out of the property and drop it. The probe should be dropped after
        // this block.
        {
            // This happens via ptrcall, which is what's being tested.
            let _probe_reference = animation_tree.tree_root().unwrap();

            // Free `animation_tree` so the reference inside is dropped.
            animation_tree.free();

            // `probe_reference` should now be the only reference left.
            assert_eq!(0, drop_counter.load(AtomicOrdering::Acquire));
        }

        // The probe should not be leaked after `probe_reference` is dropped.
        assert_eq!(1, drop_counter.load(AtomicOrdering::Acquire));
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_return_leak failed");
    }

    ok
}
