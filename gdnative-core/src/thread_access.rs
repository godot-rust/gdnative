//! Marker types to express thread safety of Godot types.

/// Marker that indicates that a value currently only has a
/// single unique reference.
pub struct Unique;

/// Marker that indicates that a value currently might be shared in the same or
/// over multiple threads.
///
/// Using this marker causes the type to be `!Send + !Sync`.
pub struct Shared(std::marker::PhantomData<*const ()>);

/// Trait to parametrise over the access markers [`Unique`](struct.Unique.html)
/// and [`Shared`](struct.Shared.html).
pub trait ThreadAccess: private::Sealed {}

impl ThreadAccess for Unique {}
impl private::Sealed for Unique {}

impl ThreadAccess for Shared {}
impl private::Sealed for Shared {}

mod private {
    pub trait Sealed {}
}
