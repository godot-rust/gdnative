use gdnative::api::Node;
use gdnative::*;

pub trait NodeExt {
    /// Gets a node at `path`, assumes that it's safe to use, and casts it to `T`.
    ///
    /// # Safety
    ///
    /// See `Ptr::assume_safe`.
    unsafe fn get_typed_node<T: GodotObject, P: Into<NodePath>>(
        &self,
        path: P,
    ) -> TRef<'_, T, thread_access::Shared>;
}

impl NodeExt for Node {
    unsafe fn get_typed_node<T: GodotObject, P: Into<NodePath>>(
        &self,
        path: P,
    ) -> TRef<'_, T, thread_access::Shared> {
        self.get_node(path.into())
            .expect("node should exist")
            .assume_safe()
            .cast()
            .expect("node should be of the correct type")
    }
}
