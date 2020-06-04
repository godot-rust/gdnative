use crate::{Basis, Vector3};
use euclid::{default, Point3D, Transform3D, UnknownUnit};

/// 3D Transformation (3x4 matrix) Using basis + origin representation.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
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

    /// Creates a `Basis` from the rotation and scaling of the provided transform.
    #[inline]
    pub fn from_transform(transform: &default::Transform3D<f32>) -> Transform {
        Self::from_typed_transform::<UnknownUnit, UnknownUnit>(transform)
    }

    /// Creates a `Basis` from the rotation and scaling of the provided transform, in `Dst` space.
    #[inline]
    pub fn from_typed_transform<Src, Dst>(transform: &Transform3D<f32, Src, Dst>) -> Transform {
        Transform {
            basis: Basis::from_typed_transform(transform),
            origin: transform
                .transform_point3d(Point3D::origin())
                .unwrap_or_else(Point3D::origin)
                .to_vector()
                .to_untyped(),
        }
    }
}
