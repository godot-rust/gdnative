use sys;
use get_api;
use GodotType;
use GodotString;
use Variant;
use std::fmt;

/// A reference-counted relative or absolute path in a scene tree, for use with `Node.get_node()` and similar
/// functions. It can reference a node, a resource within a node, or a property of a node or
/// resource.
///
/// `"Path2D/PathFollow2D/Sprite:texture:size"` would refer to the size property of the texture
/// resource on the node named “Sprite” which is a child of the other named nodes in the path.
/// Note that if you want to get a resource, you must end the path with a colon,
/// otherwise the last element will be used as a property name.
///
/// If a string is passed to `Node.get_node()`, it will be automatically converted to a `NodePath`,
/// but `NodePath` can be parsed ahead of time with `NodePath::from_str()` or `NodePath::from_godot_string()`.
///
/// A `NodePath` consists of node names, “subnode” (resource) names, and the name of a property in
/// the final node or resource.
pub struct NodePath(pub(crate) sys::godot_node_path);

impl NodePath {
    /// Create a `NodePath` from a string, e.g. `"Path2D/PathFollow2D/Sprite:texture:size"`.
    /// A path is absolute if it starts with a slash. Absolute paths are only valid in the
    /// global scene tree, not within individual scenes. In a relative path, `"."` and `".."`
    /// indicate the current node and its parent.
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

    /// Create a `NodePath` from a GodotString.
    pub fn from_godot_string(path: &GodotString) -> Self {
        unsafe {
            let mut dest = sys::godot_node_path::default();
            (get_api().godot_node_path_new)(&mut dest, &path.0);
            NodePath(dest)
        }
    }

    /// Returns `true` if the node path is empty.
    pub fn is_empty(&self) -> bool {
        unsafe {
            (get_api().godot_node_path_is_empty)(&self.0)
        }
    }

    /// Returns `true` if the node path is absolute.
    pub fn is_absolute(&self) -> bool {
        unsafe {
            (get_api().godot_node_path_is_absolute)(&self.0)
        }
    }

    // TODO DOC
    pub fn name_count(&mut self) -> i32 {
        unsafe {
            (get_api().godot_node_path_get_name_count)(&mut self.0)
        }
    }

    /// Returns the resource name of the specified `idx`, 0 to subname_count()
    pub fn subname(&self, idx: i32) -> GodotString {
        unsafe {
            GodotString((get_api().godot_node_path_get_subname)(&self.0, idx))
        }
    }

    /// Returns the number of resource names in the path.
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

    /// Returns the `NodePath` as a `GodotString`
    pub fn to_godot_string(&self) -> GodotString {
        unsafe {
            GodotString((get_api().godot_node_path_as_string)(&self.0))
        }
    }

    /// Returns the `NodePath` as a `String`
    pub fn to_string(&self) -> String {
        self.to_godot_string().to_string()
    }

    impl_common_methods! {
        /// Creates a new reference to this node path.
        pub fn new_ref(&self) -> NodePath : godot_node_path_new_copy;
    }
}

impl_basic_traits!(
    for NodePath as godot_node_path {
        Drop => godot_node_path_destroy;
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
