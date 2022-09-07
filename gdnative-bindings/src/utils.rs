//! Utility functions and extension traits that depend on generated bindings

use gdnative_core::core_types::NodePath;
use gdnative_core::export::NativeClass;
use gdnative_core::object::{SubClass, TInstance, TRef};

use super::generated::{Engine, Node, SceneTree};

/// Convenience method  to obtain a reference to an "auto-load" node, that is a child of the root
/// node.
///
/// Returns `None` if the node does not exist or is not of the correct type.
///
/// # Safety
///
/// This method accesses the scene tree. As a result, any calls to this function must
/// follow the official [thread-safety guidelines][thread-safety]. `assume_safe`
/// invariants must be observed for the resulting node during `'a`, if any.
///
/// [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
pub unsafe fn autoload<'a, T>(name: &str) -> Option<TRef<'a, T>>
where
    T: SubClass<Node>,
{
    Engine::godot_singleton()
        .get_main_loop()?
        .assume_safe()
        .cast::<SceneTree>()?
        .root()?
        .assume_safe()
        .get_node(name)?
        .assume_safe()
        .cast::<T>()
}

pub trait NodeResolveExt<P: Into<NodePath>> {
    /// Convenience method to obtain a reference to a node at `path` relative to `self`,
    /// and cast it to the desired type. Returns `None` if the node does not exist or is
    /// not of the correct type.
    ///
    /// # Safety
    ///
    /// This method accesses the scene tree. As a result, any calls to this function must
    /// follow the official [thread-safety guidelines][thread-safety]. `assume_safe`
    /// invariants must be observed for the resulting node during `'a`, if any.
    ///
    /// [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
    unsafe fn get_node_as<'a, T>(&self, path: P) -> Option<TRef<'a, T>>
    where
        T: SubClass<Node>;

    /// Convenience method to obtain a reference to a node at `path` relative to `self`,
    /// and cast it to an instance of the desired `NativeClass` type. Returns `None` if
    /// the node does not exist or is not of the correct type.
    ///
    /// # Safety
    ///
    /// This method accesses the scene tree. As a result, any calls to this function must
    /// follow the official [thread-safety guidelines][thread-safety]. `assume_safe`
    /// invariants must be observed for the resulting node during `'a`, if any.
    ///
    /// [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
    unsafe fn get_node_as_instance<'a, T>(&self, path: P) -> Option<TInstance<'a, T>>
    where
        T: NativeClass,
        T::Base: SubClass<Node>,
    {
        self.get_node_as::<T::Base>(path)?.cast_instance()
    }
}

impl<N: SubClass<Node>, P: Into<NodePath>> NodeResolveExt<P> for N {
    unsafe fn get_node_as<'a, T>(&self, path: P) -> Option<TRef<'a, T>>
    where
        T: SubClass<Node>,
    {
        self.upcast().get_node(path)?.assume_safe().cast()
    }
}
