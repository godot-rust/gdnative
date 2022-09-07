//! Port of selected GDScript built-in functions.
//!
//! This module contains _some_ of the functions available in the [@GDScript] documentation.
//!
//! Reasons why a GDScript function may _not_ be ported to Rust include:
//! * they are in the Rust standard library (`abs`, `sin`, `floor`, `assert`, ...)
//! * they are already part of a godot-rust API
//!   * `print` -> [`godot_print!`][crate::log::godot_print!]
//!   * `instance_from_id` -> [`GodotObject::from_instance_id()`][crate::object::GodotObject::from_instance_id]
//!   * ...
//! * they have a private implementation, i.e. a Rust port would have different semantics
//!   * `randi`, `randf` etc. -- users should use `rand` crate
//!   * `str2var`, `bytes2var`, `hash` etc -- to be verified
//!
//! This above list is not a definitive inclusion/exclusion criterion, just a rough guideline.
//!
//! Other noteworthy special cases:
//! * GDScript `fmod` corresponds to Rust's `%` operator on `f32` (also known as the `Rem` trait).
//!
//! [@GDScript]: https://docs.godotengine.org/en/stable/classes/class_@gdscript.html

use crate::api::{Resource, ResourceLoader};
use crate::core_types::NodePath;
use crate::object::{memory::RefCounted, GodotObject, Ref, SubClass};

#[doc(inline)]
pub use gdnative_core::globalscope::*;

/// Loads a resource from the filesystem located at `path`.
///
/// The resource is loaded on the method call (unless it's referenced already elsewhere, e.g. in another script or in the scene),
/// which might cause slight delay, especially when loading scenes.
///
/// If the resource cannot be loaded, or is not of type `T` or inherited, this method returns `None`.
///
/// This method is a simplified version of [`ResourceLoader::load()`][crate::api::ResourceLoader::load],
/// which can be used for more advanced scenarios.
///
/// # Note:
/// Resource paths can be obtained by right-clicking on a resource in the Godot editor (_FileSystem_ dock) and choosing "Copy Path",
/// or by dragging the file from the _FileSystem_ dock into the script.
///
/// The path must be absolute (typically starting with `res://`), a local path will fail.
///
/// # Example:
/// Loads a scene called `Main` located in the `path/to` subdirectory of the Godot project and caches it in a variable.
/// The resource is directly stored with type `PackedScene`.
///
/// ```no_run
/// use gdnative::prelude::*;
///
/// let scene = load::<PackedScene>("res://path/to/Main.tscn").unwrap();
/// ```
#[inline]
pub fn load<T>(path: impl Into<NodePath>) -> Option<Ref<T>>
where
    T: SubClass<Resource> + GodotObject<Memory = RefCounted>,
{
    let type_hint = T::class_name();
    ResourceLoader::godot_singleton()
        .load(path.into(), type_hint, false)
        .and_then(|res| res.cast::<T>())
}
