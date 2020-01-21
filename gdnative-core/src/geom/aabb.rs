use crate::Vector3;

/// Axis-aligned bounding box.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Aabb {
    pub position: Vector3,
    pub size: Vector3,
}

impl Aabb {
    #[doc(hidden)]
    pub fn sys(&self) -> *const sys::godot_aabb {
        unsafe { std::mem::transmute::<*const Aabb, *const sys::godot_aabb>(self as *const _) }
    }

    #[doc(hidden)]
    pub fn from_sys(c: sys::godot_aabb) -> Self {
        unsafe { std::mem::transmute::<sys::godot_aabb, Self>(c) }
    }
}
