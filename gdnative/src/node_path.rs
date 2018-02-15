use sys;
use get_api;

pub struct NodePath(pub(crate) sys::godot_node_path);

impl NodePath {
    pub fn new(path: &str) -> NodePath {
        unsafe {
            let mut dest = sys::godot_node_path::default();
            let api = get_api();
            let mut from = (api.godot_string_chars_to_utf8_with_len)(path.as_ptr() as *const _, path.len() as _);
            (api.godot_node_path_new)(&mut dest, &from);
            (api.godot_string_destroy)(&mut from);
            NodePath(dest)
        }
    }
}

impl_basic_traits!(
    for NodePath as godot_node_path {
        Drop => godot_node_path_destroy;
        Clone => godot_node_path_new_copy;
        PartialEq => godot_node_path_operator_equal;
        Default => default;
    }
);
