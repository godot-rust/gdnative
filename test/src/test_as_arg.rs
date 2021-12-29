use gdnative::derive::{methods, NativeClass};
use gdnative::prelude::NodeResolveExt;
use gdnative::prelude::*;
use std::ops::Deref;

pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<MyNode>();
}

pub(crate) fn run_tests() -> bool {
    println!(" -- test_as_arg");

    let ok = std::panic::catch_unwind(|| {
        println!("   -- test_ref_as_arg");
        test_ref_as_arg();

        println!("   -- test_instance_as_arg");
        test_instance_as_arg();
    })
    .is_ok();

    if !ok {
        godot_error!("   !! Test test_as_arg failed");
    }

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

fn test_ref_as_arg() {
    // Ref<T, Unique>
    add_node_with(|n: Ref<Node2D, Unique>| n);

    // Ref<T, Shared>
    add_node_with(|n: Ref<Node2D, Unique>| n.into_shared());

    // &Ref<T, Shared>
    let mut keeper = Node2D::new().into_shared(); // keep Ref<T, Shared> alive so we can return a reference to it
    add_node_with(|n: Ref<Node2D, Unique>| {
        keeper = n.into_shared();
        &keeper
    });

    // TRef<T, Shared>
    add_node_with(|n: Ref<Node2D, Unique>| unsafe { n.into_shared().assume_safe() });
}

fn test_instance_as_arg() {
    // Instance<T, Unique>
    add_instance_with(|n: Instance<MyNode, Unique>| n);

    // Instance<T, Shared>
    add_instance_with(|n: Instance<MyNode, Unique>| n.into_shared());

    // &Instance<T, Shared>
    let mut keeper = MyNode { secret: "" }.emplace().into_shared(); // keep Instance<T, Shared> alive so we can return a reference to it
    add_instance_with(|n: Instance<MyNode, Unique>| {
        keeper = n.into_shared();
        &keeper
    });

    // TInstance<T, Shared>
    add_instance_with(|n: Instance<MyNode, Unique>| unsafe { n.into_shared().assume_safe() });
}

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
        .expect("found.map()")
}
