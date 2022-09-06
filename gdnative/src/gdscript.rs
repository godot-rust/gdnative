use gdnative_bindings::{Resource, ResourceLoader};
use gdnative_core::{
    core_types::NodePath,
    object::{memory::RefCounted, GodotObject, Ref, SubClass},
};

/// Loads a resource from the filesystem located at `path`. The resource is loaded on the method call (unless it's referenced already elsewhere, e.g. in another script or in the scene), which might cause slight delay, especially when loading scenes.
/// # Note:
/// Resource paths can be obtained by right-clicking on a resource in the FileSystem dock and choosing "Copy Path" or by dragging the file from the FileSystem dock into the script.
///
/// Load a scene called main located in the root of the project directory and cache it in a variable. var main = load("res://main.tscn")  main will contain a PackedScene resource.
/// # Important:
/// The path must be absolute, a local path will just return null.
/// This method is a simplified version of `ResourceLoader.load()`, which can be used for more advanced scenarios.
/// # Examples:
/// ```no_run
/// use gdnative::globalscope::load;
/// use gdnative::prelude::PackedScene;
///
/// let scene = load::<PackedScene>("res://path").unwrap();
/// ```
#[inline]
pub fn load<T>(path: impl Into<NodePath>) -> Option<Ref<T>>
where
    T: SubClass<Resource> + GodotObject<Memory = RefCounted>,
{
    ResourceLoader::godot_singleton()
        .load(path.into(), "", false)
        .unwrap()
        .cast::<T>()
}
