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

/// Trait to parametrize over the ownership markers [`Unique`], [`Shared`] and [`ThreadLocal`].
///
/// This trait is sealed and has no public members.
///
/// It specifies the ownership policy of godot-rust smart pointers such as [`Ref`][super::Ref].
/// Ownership specifies how references _own_ an object, i.e. how they point to it and who is responsible
/// for its destruction (in case of [`RefCounted`][super::memory::RefCounted]). Furthermore, it defines
/// from where the object can be accessed, and if sharing the object across threads is possible.
pub trait Ownership: private::Sealed {}

/// Trait to parametrize over the ownership markers that are local to the current thread:
/// [`Unique`] and [`ThreadLocal`].
pub trait LocalThreadOwnership: Ownership {}

/// Trait to parametrize over the ownership markers that are not unique:
/// [`Shared`] and [`ThreadLocal`].
pub trait NonUniqueOwnership: Ownership {}

impl Ownership for Unique {}
impl LocalThreadOwnership for Unique {}
impl private::Sealed for Unique {}

impl Ownership for Shared {}
impl NonUniqueOwnership for Shared {}
impl private::Sealed for Shared {}

impl Ownership for ThreadLocal {}
impl LocalThreadOwnership for ThreadLocal {}
impl NonUniqueOwnership for ThreadLocal {}
impl private::Sealed for ThreadLocal {}

mod private {
    pub trait Sealed {}
}
