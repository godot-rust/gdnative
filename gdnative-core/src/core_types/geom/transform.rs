use std::ops::Mul;

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

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Self = Self {
        basis: Basis::IDENTITY,
        origin: Vector3::ZERO,
    };

    /// Creates a new transform from its three basis vectors and origin.
    #[inline]
    pub fn from_axis_origin(
        x_axis: Vector3,
        y_axis: Vector3,
        z_axis: Vector3,
        origin: Vector3,
    ) -> Self {
        Self {
            origin,
            basis: Basis::from_elements([x_axis, y_axis, z_axis]),
        }
    }

    /// Returns this transform, with its origin moved by a certain `translation`
    #[inline]
    pub fn translated(&self, translation: Vector3) -> Self {
        Self {
            origin: self.origin + translation,
            basis: self.basis,
        }
    }

    /// Returns a vector transformed (multiplied) by the matrix.
    #[inline]
    pub fn xform(&self, v: Vector3) -> Vector3 {
        self.basis.xform(v) + self.origin
    }

    /// Returns a vector transformed (multiplied) by the transposed basis
    /// matrix.
    ///
    /// **Note:** This results in a multiplication by the inverse of the matrix
    /// only if it represents a rotation-reflection.
    #[inline]
    pub fn xform_inv(&self, v: Vector3) -> Vector3 {
        self.basis.xform_inv(v - self.origin)
    }

    /// Returns the inverse of the transform, under the assumption that the
    /// transformation is composed of rotation and translation (no scaling, use
    /// affine_inverse for transforms with scaling).
    #[inline]
    pub fn inverse(&self) -> Self {
        let basis_inv = self.basis.transposed();
        let origin_inv = basis_inv.xform(-self.origin);
        Self {
            origin: origin_inv,
            basis: basis_inv,
        }
    }

    /// Returns the inverse of the transform, under the assumption that the
    /// transformation is composed of rotation, scaling and translation.
    #[inline]
    pub fn affine_inverse(&self) -> Self {
        let basis_inv = self.basis.inverted();
        let origin_inv = basis_inv.xform(-self.origin);
        Self {
            origin: origin_inv,
            basis: basis_inv,
        }
    }

    /// Returns a copy of the transform rotated such that its -Z axis points
    /// towards the target position.
    ///
    /// The transform will first be rotated around the given up vector, and then
    /// fully aligned to the target by a further rotation around an axis
    /// perpendicular to both the target and up vectors.
    #[inline]
    pub fn looking_at(&self, target: Vector3, up: Vector3) -> Self {
        let up = up.normalized();
        let v_z = (self.origin - target).normalized();
        let v_x = up.cross(v_z);
        let v_y = v_z.cross(v_x);

        Transform {
            basis: Basis::from_elements([v_x, v_y, v_z]).transposed(),
            origin: self.origin,
        }
    }

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
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    #[inline]
    fn mul(self, rhs: Transform) -> Self::Output {
        let origin = self.xform(rhs.origin);
        let basis = self.basis * rhs.basis;
        Transform { origin, basis }
    }
}
