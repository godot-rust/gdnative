use crate::Vector3;

/// Plane in hessian form.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Plane {
    pub normal: Vector3,
    pub d: f32,
}

impl Plane {
    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_plane {
        unsafe { std::mem::transmute::<*const Plane, *const sys::godot_plane>(self as *const _) }
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(c: sys::godot_plane) -> Self {
        unsafe { std::mem::transmute::<sys::godot_plane, Self>(c) }
    }
}
