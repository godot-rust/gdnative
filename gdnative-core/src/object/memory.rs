//! Marker types to express the memory management method of Godot types.
// Note: markers are enums to prevent instantiation and avoid derive (empty/non-inhabitable types)

use crate::object::bounds::MemorySpec;

/// Marker that indicates that a type is manually managed.
pub enum ManuallyManaged {}

/// Marker that indicates that a type is reference-counted.
pub enum RefCounted {}

/// Trait to parameterize over the memory management markers [`ManuallyManaged`] and [`RefCounted`].
///
/// This trait is sealed and has no public members.
///
/// It defines how memory is managed for Godot objects in smart pointers, for example [`Ref`][super::Ref].
/// Generally, classes inheriting `Reference` are ref-counted, while the rest (i.e. everything inheriting
/// `Object` which is not a `Reference`) is manually managed.
pub trait Memory: MemorySpec + private::Sealed {}

impl Memory for ManuallyManaged {}
impl private::Sealed for ManuallyManaged {}

impl Memory for RefCounted {}
impl private::Sealed for RefCounted {}

mod private {
    pub trait Sealed {}
}
