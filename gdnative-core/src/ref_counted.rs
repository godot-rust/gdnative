/// A trait for incrementing the reference count to a Godot object.
pub trait RefCounted {
    /// Creates a new reference to the underlying object.
    fn new_ref(&self) -> Self;
}
