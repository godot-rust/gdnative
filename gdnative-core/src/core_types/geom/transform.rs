use std::ops::Mul;

use crate::core_types::{Basis, Vector3};

/// Affine 3D transform (3x4 matrix).
///
/// Used for 3D linear transformations. Uses a basis + origin representation.
/// The
///
/// Expressed as a 3x4 matrix, this transform consists of 3 basis (column) vectors `a`, `b`, `c`
/// as well as an origin `o`; more information in [`Self::from_basis_origin()`]:
/// ```text
/// [ a.x  b.x  c.x  o.x ]
/// [ a.y  b.y  c.y  o.y ]
/// [ a.z  b.z  c.z  o.z ]
/// ```
///
/// See also [Transform](https://docs.godotengine.org/en/stable/classes/class_transform.html) in the Godot API doc.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transform {
    /// The basis is a matrix containing 3 vectors as its columns. They can be interpreted
    /// as the basis vectors of the transformed coordinate system.
    pub basis: Basis,

    /// The new origin of the transformed coordinate system.
    pub origin: Vector3,
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// Identity transform; leaves objects unchanged when applied.
    pub const IDENTITY: Self = Self {
        basis: Basis::IDENTITY,
        origin: Vector3::ZERO,
    };

    /// Transform that mirrors along the **X axis** (perpendicular to the YZ plane).
    pub const FLIP_X: Self = Self {
        basis: Basis::FLIP_X,
        origin: Vector3::ZERO,
    };

    /// Transform that mirrors along the **Y axis** (perpendicular to the XZ plane).
    pub const FLIP_Y: Self = Self {
        basis: Basis::FLIP_Y,
        origin: Vector3::ZERO,
    };

    /// Transform that mirrors along the **Z axis** (perpendicular to the XY plane).
    pub const FLIP_Z: Self = Self {
        basis: Basis::FLIP_Z,
        origin: Vector3::ZERO,
    };

    /// Creates a new transform from three basis vectors and the coordinate system's origin.
    ///
    /// Each vector represents a basis vector in the *transformed* coordinate system.
    /// For example, `a` is the result of transforming the X unit vector `(1, 0, 0)`.
    /// The 3 vectors need to be linearly independent.
    ///
    /// Basis vectors are stored as column vectors in the matrix, see also [`Basis::from_basis_vectors()`].
    ///
    /// The construction `Transform::from_basis_origin(a, b, c, o)` will create the following 3x4 matrix:
    /// ```text
    /// [ a.x  b.x  c.x  o.x ]
    /// [ a.y  b.y  c.y  o.y ]
    /// [ a.z  b.z  c.z  o.z ]
    /// ```
    #[inline]
    pub const fn from_basis_origin(
        basis_vector_a: Vector3,
        basis_vector_b: Vector3,
        basis_vector_c: Vector3,
        origin: Vector3,
    ) -> Self {
        Self {
            basis: Basis::from_basis_vectors(basis_vector_a, basis_vector_b, basis_vector_c),
            origin,
        }
    }

    /// Returns this transform, with its origin moved by a certain `translation`
    #[inline]
    pub fn translated(&self, translation: Vector3) -> Self {
        Self {
            basis: self.basis,
            origin: self.origin + translation,
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
            basis: basis_inv,
            origin: origin_inv,
        }
    }

    /// Returns the inverse of the transform, under the assumption that the
    /// transformation is composed of rotation, scaling and translation.
    #[inline]
    pub fn affine_inverse(&self) -> Self {
        let basis_inv = self.basis.inverted();
        let origin_inv = basis_inv.xform(-self.origin);

        Self {
            basis: basis_inv,
            origin: origin_inv,
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

        Self {
            basis: Basis::from_rows(v_x, v_y, v_z).transposed(),
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
    fn mul(self, rhs: Self) -> Self::Output {
        let origin = self.xform(rhs.origin);
        let basis = self.basis * rhs.basis;
        Self { origin, basis }
    }
}
