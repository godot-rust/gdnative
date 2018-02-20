use sys;
use get_api;
use GodotType;
use GodotString;
use Variant;
use std::fmt;

pub struct NodePath(pub(crate) sys::godot_node_path);

impl NodePath {
    pub fn from_str(path: &str) -> Self {
        unsafe {
            let mut dest = sys::godot_node_path::default();
            let api = get_api();
            let mut from = (api.godot_string_chars_to_utf8_with_len)(path.as_ptr() as *const _, path.len() as _);
            (api.godot_node_path_new)(&mut dest, &from);
            (api.godot_string_destroy)(&mut from);
            NodePath(dest)
        }
    }

    pub fn from_godot_string(path: &GodotString) -> Self {
        unsafe {
            let mut dest = sys::godot_node_path::default();
            (get_api().godot_node_path_new)(&mut dest, &path.0);
            NodePath(dest)
        }
    }

    pub fn is_empty(&self) -> bool {
        unsafe {
            (get_api().godot_node_path_is_empty)(&self.0)
        }
    }

    pub fn is_absolute(&self) -> bool {
        unsafe {
            (get_api().godot_node_path_is_absolute)(&self.0)
        }
    }

    pub fn subname(&self, idx: i32) -> GodotString {
        unsafe {
            GodotString((get_api().godot_node_path_get_subname)(&self.0, idx))
        }
    }

    pub fn subname_count(&self) -> i32 {
        unsafe {
            (get_api().godot_node_path_get_subname_count)(&self.0)
        }
    }

    pub fn concatenated_subnames(&self) -> GodotString {
        unsafe {
            GodotString((get_api().godot_node_path_get_concatenated_subnames)(&self.0))
        }
    }

    pub fn to_godot_string(&self) -> GodotString {
        unsafe {
            GodotString((get_api().godot_node_path_as_string)(&self.0))
        }
    }

    pub fn to_string(&self) -> String {
        self.to_godot_string().to_string()
    }
}

impl_basic_traits!(
    for NodePath as godot_node_path {
        Drop => godot_node_path_destroy;
        Clone => godot_node_path_new_copy;
        Eq => godot_node_path_operator_equal;
    }
);

impl GodotType for NodePath {
    fn to_variant(&self) -> Variant { Variant::from_node_path(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.try_to_node_path() }
}

impl fmt::Debug for NodePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "NodePath({})", self.to_string())
    }
}
