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

impl Clone for NodePath {
    fn clone(&self) -> NodePath {
        unsafe {
            let mut dest = sys::godot_node_path::default();
            (get_api().godot_node_path_new_copy)(&mut dest, &self.0);
            NodePath(dest)
        }
    }
}

impl Drop for NodePath {
    fn drop(&mut self) {
        unsafe {
            (get_api().godot_node_path_destroy)(&mut self.0);
        }
    }
}
