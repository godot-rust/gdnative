//! Marker types to express the memory management method of Godot types.

use crate::object::RefKindSpec;

/// Marker that indicates that a type is manually managed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct ManuallyManaged;

/// Marker that indicates that a type is reference-counted.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct RefCounted;

/// Trait to parameterize over the memory management markers
/// [`ManuallyManaged`](struct.ManuallyManaged.html) and [`RefCounted`](struct.RefCounted.html).
///
/// This trait is sealed and has no public members.
pub trait RefKind: RefKindSpec + private::Sealed {}

impl RefKind for ManuallyManaged {}
impl private::Sealed for ManuallyManaged {}

impl RefKind for RefCounted {}
impl private::Sealed for RefCounted {}

mod private {
    pub trait Sealed {}
}
