use crate::core_types::Vector3;

/// Axis-aligned bounding box.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Aabb {
    pub position: Vector3,
    pub size: Vector3,
}

impl Aabb {
    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_aabb {
        unsafe { std::mem::transmute::<*const Aabb, *const sys::godot_aabb>(self as *const _) }
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(c: sys::godot_aabb) -> Self {
        unsafe { std::mem::transmute::<sys::godot_aabb, Self>(c) }
    }
}
