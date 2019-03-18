use std::mem;
use std::ops::{Deref, DerefMut};

/// Manually managed Godot classes implementing `free`.
pub trait Free {
    unsafe fn godot_free(self);
}

/// Manually managed Godot classes implementing `queue_free`.
pub trait QueueFree {
    unsafe fn godot_queue_free(&mut self);
}

/// A wrapper that automatically frees the object when dropped.
pub struct FreeOnDrop<T: Free + Clone> {
    ptr: T,
}

impl<T> FreeOnDrop<T>  where T: Free + Clone {

    pub unsafe fn new(ptr: T) -> Self {
        FreeOnDrop { ptr }
    }

    pub fn forget(self) -> T {
        let ptr = self.ptr.clone();
        mem::forget(self);

        ptr
    }
}

impl<T> Drop for FreeOnDrop<T> where T: Free + Clone {
    fn drop(&mut self) {
        unsafe { self.ptr.clone().godot_free(); }
    }
}

impl<T> Deref for FreeOnDrop<T> where T: Free + Clone {
    type Target = T;
    fn deref(&self) -> &T { &self.ptr }
}

impl<T> DerefMut for FreeOnDrop<T> where T: Free + Clone {
    fn deref_mut(&mut self) -> &mut T { &mut self.ptr }
}

/// A wrapper that automatically enqueues the object for deletion when dropped.
pub struct QueueFreeOnDrop<T: QueueFree + Clone> {
    ptr: T,
}

impl<T> QueueFreeOnDrop<T>  where T: QueueFree + Clone {

    pub unsafe fn new(ptr: T) -> Self {
        QueueFreeOnDrop { ptr }
    }

    pub fn forget(self) -> T {
        let ptr = self.ptr.clone();
        mem::forget(self);

        ptr
    }
}

impl<T> Drop for QueueFreeOnDrop<T> where T: QueueFree + Clone {
    fn drop(&mut self) {
        unsafe { self.ptr.godot_queue_free(); }
    }
}

impl<T> Deref for QueueFreeOnDrop<T> where T: QueueFree + Clone{
    type Target = T;
    fn deref(&self) -> &T { &self.ptr }
}

impl<T> DerefMut for QueueFreeOnDrop<T> where T: QueueFree + Clone {
    fn deref_mut(&mut self) -> &mut T { &mut self.ptr }
}
