use gdnative::api;
use gdnative::export::StaticallyNamed;
use gdnative::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_return_leak();

    status
}

pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<Probe>();
}

/// Probe type to track dropping of the Reference it's attached to. As the leak
/// only happens with ptrcalls, this will have to inherit a Reference class that is
/// accepted somewhere in the Godot API as a property. Here, the chosen type is
/// `AnimationNodeAdd2`, of the property `AnimationTree::tree_root`.
struct Probe {
    drop_count: Arc<AtomicUsize>,
}

impl NativeClass for Probe {
    type Base = api::AnimationNodeAdd2;
    type UserData = user_data::RwLockData<Probe>;

    fn nativeclass_register_properties(_builder: &ClassBuilder<Self>) {}
}

impl StaticallyNamed for Probe {
    const CLASS_NAME: &'static str = "ReturnLeakProbe";
}

impl Drop for Probe {
    fn drop(&mut self) {
        self.drop_count.fetch_add(1, AtomicOrdering::AcqRel);
    }
}

#[methods]
impl Probe {}

crate::godot_itest! { test_return_leak {
    let drop_counter = Arc::new(AtomicUsize::new(0));

    // The object used for its ptrcall getter
    let animation_tree = api::AnimationTree::new();

    // Create an instance of the probe, and drop the reference after setting the property
    // to it. After this block, the only reference should be the one in `animation_tree`.
    {
        let probe = Instance::emplace(Probe {
            drop_count: Arc::clone(&drop_counter),
        });
        let base = probe.into_base().into_shared();
        animation_tree.set_tree_root(base);
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
}}
