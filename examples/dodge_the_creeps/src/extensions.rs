use gdnative::*;

pub trait NodeExt {
    unsafe fn get_typed_node<T: GodotObject, P: Into<NodePath>>(&self, path: P) -> Option<T>;
}

macro_rules! impl_node_ext_for {
    ($t:ty) => {
        impl NodeExt for $t {
            unsafe fn get_typed_node<T: GodotObject, P: Into<NodePath>>(
                &self,
                path: P,
            ) -> Option<T> {
                self.get_node(path.into()).and_then(|node| node.cast())
            }
        }
    };
}

impl_node_ext_for!(Node);
impl_node_ext_for!(RigidBody2D);
impl_node_ext_for!(CanvasLayer);
impl_node_ext_for!(Area2D);
