//! Typestates to express ownership and thread safety of Godot types.

/// Marker that indicates that a value currently only has a
/// single unique reference.
///
/// Using this marker causes the type to be `!Sync`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Unique(std::marker::PhantomData<*const ()>);
unsafe impl Send for Unique {}

/// Marker that indicates that a value currently might be shared in the same or
/// over multiple threads.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Shared;

/// Marker that indicates that a value can currently only be shared in the same thread.
///
/// Using this marker causes the type to be `!Send + !Sync`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct ThreadLocal(std::marker::PhantomData<*const ()>);

/// Trait to parametrise over the access markers [`Unique`](struct.Unique.html),
/// [`Shared`](struct.Shared.html), and [`ThreadLocal`](struct.ThreadLocal.html).
///
/// This trait is sealed and has no public members.
pub trait ThreadAccess: private::Sealed {}

/// Trait to parametrise over the access markers that are local to the current thread:
/// [`Unique`](struct.Unique.html) and [`ThreadLocal`](struct.ThreadLocal.html).
pub trait LocalThreadAccess: ThreadAccess {}

/// Trait to parametrise over the access markers that are not unique:
/// [`Shared`](struct.Shared.html) and [`ThreadLocal`](struct.ThreadLocal.html).
pub trait NonUniqueThreadAccess: ThreadAccess {}

impl ThreadAccess for Unique {}
impl LocalThreadAccess for Unique {}
impl private::Sealed for Unique {}

impl ThreadAccess for Shared {}
impl NonUniqueThreadAccess for Shared {}
impl private::Sealed for Shared {}

impl ThreadAccess for ThreadLocal {}
impl LocalThreadAccess for ThreadLocal {}
impl NonUniqueThreadAccess for ThreadLocal {}
impl private::Sealed for ThreadLocal {}

mod private {
    pub trait Sealed {}
}
