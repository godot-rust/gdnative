use crate::core_types::{Basis, Vector3};

/// 3D Transformation (3x4 matrix) Using basis + origin representation.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transform {
    /// The basis is a matrix containing 3 Vector3 as its columns: X axis, Y axis, and Z axis.
    /// These vectors can be interpreted as the basis vectors of local coordinate system
    /// traveling with the object.
    pub basis: Basis,
    /// The translation offset of the transform.
    pub origin: Vector3,
}

impl Transform {
    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_transform {
        unsafe {
            std::mem::transmute::<*const Transform, *const sys::godot_transform>(self as *const _)
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(c: sys::godot_transform) -> Self {
        unsafe { std::mem::transmute::<sys::godot_transform, Self>(c) }
    }

    #[inline]
    pub fn translate(origin: Vector3) -> Transform {
        Transform {
            basis: Basis::identity(),
            origin,
        }
    }
}
