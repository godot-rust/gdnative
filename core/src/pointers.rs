use std::mem;
use std::ops::*;
use sys;
use get_api;
use GodotObject;

#[doc(hidden)]
pub unsafe trait UnsafeObject : GodotObject {}

pub struct Owned<T: UnsafeObject> {
    ptr: T
}

/// An owned reference to a manually managed Godot class.
///
/// The object is automatically freed when the `Owned<T>` is dropped.
impl<T> Owned<T> where T: UnsafeObject {
    pub fn get(&self) -> &T {
        &self.ptr
    }

    pub fn forget(self) -> Unsafe<T> {
        unsafe {
            let ret = mem::transmute_copy(&self);
            mem::forget(self);

            ret
        }
    }

    pub unsafe fn get_unsafe(&self) -> Unsafe<T> {
        Unsafe { ptr:   T::from_sys(self.ptr.to_sys()) }
    }

    pub unsafe fn from_unsafe(other: Unsafe<T>) -> Self {
        Owned {
            ptr: other.ptr
        }
    }
}

impl<T: UnsafeObject> Drop for Owned<T> {
    fn drop(&mut self) {
        unsafe {
           (get_api().godot_object_destroy)(self.ptr.to_sys());
        }
    }
}

impl<T: UnsafeObject> Deref for Owned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

/// An unsafe reference to a manually managed Godot class.
pub struct Unsafe<T: UnsafeObject> {
    ptr: T
}

impl<T> Unsafe<T> where T: UnsafeObject {
    pub fn get(&self) -> &T {
        &self.ptr
    }

    pub unsafe fn free(self) {
        (get_api().godot_object_destroy)(self.ptr.to_sys())
    }
}
