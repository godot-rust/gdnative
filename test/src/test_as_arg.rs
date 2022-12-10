use gdnative::derive::{methods, NativeClass};
use gdnative::prelude::NodeResolveExt;
use gdnative::prelude::*;
use std::mem::MaybeUninit;
use std::ops::Deref;

#[cfg(not(feature = "no-manual-register"))]
pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<MyNode>();
}

#[cfg(feature = "no-manual-register")]
pub(crate) fn register(_handle: InitHandle) {}

pub(crate) fn run_tests() -> bool {
    let mut ok = true;

    ok &= test_as_arg_ref();
    ok &= test_as_arg_instance();

    ok
}

// ----------------------------------------------------------------------------------------------------------------------------------------------

#[derive(NativeClass, Debug)]
#[inherit(Spatial)]
#[no_constructor]
struct MyNode {
    secret: &'static str,
}

#[methods]
impl MyNode {}

// ----------------------------------------------------------------------------------------------------------------------------------------------

crate::godot_itest! { test_as_arg_ref {
    // Ref<T, Unique>
    add_node_with(|n: Ref<Node2D, Unique>| n);

    // Ref<T, Shared>
    add_node_with(|n: Ref<Node2D, Unique>| n.into_shared());

    // &Ref<T, Shared>
    let mut keeper: MaybeUninit<Ref<Node2D, Shared>> = MaybeUninit::uninit(); // keep Ref<T, Shared> alive so we can return a reference to it
    add_node_with(|n: Ref<Node2D, Unique>| {
        keeper.write(n.into_shared());
        unsafe { keeper.assume_init_ref() }
    });

    // TRef<T, Shared>
    add_node_with(|n: Ref<Node2D, Unique>| unsafe { n.into_shared().assume_safe() });
}}

crate::godot_itest! { test_as_arg_instance {
    // Instance<T, Unique>
    add_instance_with(|n: Instance<MyNode, Unique>| n);

    // Instance<T, Shared>
    add_instance_with(|n: Instance<MyNode, Unique>| n.into_shared());

    // &Instance<T, Shared>
    let mut keeper: MaybeUninit<Instance<MyNode, Shared>> = MaybeUninit::uninit(); // keep Instance<T, Shared> alive so we can return a reference to it
    add_instance_with(|n: Instance<MyNode, Unique>| {
        keeper.write(n.into_shared());
        unsafe { keeper.assume_init_ref() }
    });

    // TInstance<T, Shared>
    add_instance_with(|n: Instance<MyNode, Unique>| unsafe { n.into_shared().assume_safe() });
}}

fn add_node_with<F, T>(to_arg: F)
where
    F: FnOnce(Ref<Node2D, Unique>) -> T,
    T: AsArg<Node>,
{
    let parent = Node::new();
    let child = Node2D::new();
    let child_id: i64 = child.get_instance_id();
    child.set_name("ch");

    let child: T = to_arg(child);

    parent.add_child(child, /*legible_unique_name*/ true);

    let found = parent.get_node("ch").expect("get_node() for Ref");
    let found_tref = unsafe { found.assume_safe() };

    assert_eq!(found_tref.get_instance_id(), child_id);
    parent.free()
}

fn add_instance_with<F, T>(to_arg: F)
where
    F: FnOnce(Instance<MyNode, Unique>) -> T,
    T: AsArg<Node>,
{
    let parent = Node::new();
    let child = MyNode { secret: "yes" }.emplace();

    let child_id: i64 = child
        .map(|_, node| {
            node.set_name("ch");
            node.get_instance_id()
        })
        .expect("child.map()");

    let child: T = to_arg(child);

    parent.add_child(child, /*legible_unique_name*/ true);

    let found: TInstance<MyNode> = unsafe {
        parent
            .deref()
            .get_node_as_instance::<MyNode>("ch")
            .expect("get_node() for Instance")
    };

    found
        .map(|user, node| {
            assert_eq!(node.get_instance_id(), child_id);
            assert_eq!(user.secret, "yes");
        })
        .expect("found.map()");
    parent.free()
}
