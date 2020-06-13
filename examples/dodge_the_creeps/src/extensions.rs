use gdnative::api::Node;
use gdnative::*;

pub trait NodeExt {
    unsafe fn get_typed_node<T: GodotObject, P: Into<NodePath>>(&self, path: P) -> Option<T>;
}

impl NodeExt for Node {
    unsafe fn get_typed_node<T: GodotObject, P: Into<NodePath>>(&self, path: P) -> Option<T> {
        self.get_node(path.into()).and_then(|node| node.cast())
    }
}
