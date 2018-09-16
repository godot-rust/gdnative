use std::mem;
use std::ops::*;
use sys;
use get_api;
use GodotObject;
use PointerType;

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
        Unsafe { ptr: T::obj_from_sys(self.ptr.obj_to_sys()) }
    }

    pub unsafe fn from_unsafe(other: Unsafe<T>) -> Self {
        Owned {
            ptr: other.ptr
        }
    }

    #[doc(hidden)]
    pub unsafe fn from_sys(ptr: *mut sys::godot_object) -> Self {
        Owned { ptr: T::obj_from_sys(ptr) }
    }
}

impl<T: UnsafeObject> Drop for Owned<T> {
    fn drop(&mut self) {
        unsafe {
           (get_api().godot_object_destroy)(self.ptr.obj_to_sys());
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
        (get_api().godot_object_destroy)(self.ptr.obj_to_sys())
    }

    #[doc(hidden)]
    pub unsafe fn to_sys(&self) -> *mut sys::godot_object {
        self.ptr.obj_to_sys()
    }

    #[doc(hidden)]
    pub unsafe fn from_sys(obj: *mut sys::godot_object) -> Self {
        Unsafe { ptr: T::obj_from_sys(obj) }
    }
}

unsafe impl<T: UnsafeObject> PointerType for Unsafe<T> {
    type Target = T;

    #[doc(hidden)]
    unsafe fn to_sys(&self) -> *mut sys::godot_object {
        self.ptr.obj_to_sys()
    }

    #[doc(hidden)]
    unsafe fn from_sys(obj: *mut sys::godot_object) -> Self {
        Unsafe { ptr: T::obj_from_sys(obj) }
    }
}
