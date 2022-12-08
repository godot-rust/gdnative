use std::ops::{Mul, MulAssign};

use crate::core_types::{Basis, Vector3};

// Note regarding naming of interpolation: there are 3 namings in Godot.
// * `lerp` + `slerp` for simple types
// * `linear_interpolate` for vectors and colors
//    (now renamed to [`lerp`](https://docs.godotengine.org/en/latest/classes/class_vector3.html?highlight=vector3#class-vector3-method-lerp) + `slerp`)
// * `Vector3` also has `cubic_interpolate` and `bezier_interpolate`, which might explain the origins
// * `interpolate_with` for transforms; in Godot 4 also
//   [`sphere_interpolate_with`](https://docs.godotengine.org/en/latest/classes/class_transform3d.html#class-transform3d-method-sphere-interpolate-with)
//
// We currently also have `Transform2D::interpolate_with()`.
// In an ideal world, all those would be called `lerp` and `slerp`.

/// Affine 3D transform (3x4 matrix).
///
/// Used for 3D linear transformations. Uses a basis + origin representation.
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
    #[deprecated = "`translated` is not relative to the transform's coordinate system \
    and thus inconsistent with GDScript. Please use translated_global() instead. \
    This method will be renamed to translated_local in gdnative 0.12."]
    #[inline]
    pub fn translated(&self, translation: Vector3) -> Self {
        self.translated_local(translation)
    }

    /// Returns this transform, with its origin moved by a certain `translation`
    #[inline]
    pub fn translated_local(&self, translation: Vector3) -> Self {
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
        let basis = self.basis.transposed();
        Transform {
            basis,
            origin: basis.xform(-self.origin),
        }
    }

    /// Returns the inverse of the transform, under the assumption that the
    /// transformation is composed of rotation, scaling and translation.
    #[inline]
    pub fn affine_inverse(&self) -> Self {
        let basis_inv = self.basis.inverse();
        let origin_inv = basis_inv.xform(-self.origin);

        Self {
            basis: basis_inv,
            origin: origin_inv,
        }
    }

    /// In-place rotation of the transform around the given axis by the given
    /// angle (in radians), using matrix multiplication. The axis must be a
    /// normalized vector.
    /// Due to nature of the operation, a new transform is created first.
    #[inline]
    pub fn rotated(&self, axis: Vector3, phi: f32) -> Self {
        Transform {
            basis: Basis::from_axis_angle(axis, phi),
            origin: Vector3::default(),
        } * (*self)
    }

    /*
    /// Returns the rotated transform around the given axis by the given angle (in radians),
    /// using matrix multiplication. The axis must be a normalized vector.
    #[inline]
    fn rotate(&mut self, axis: Vector3, phi: f32) {
        *self = self.rotated(axis, phi);
    }
    */

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

    /// Scales basis and origin of the transform by the given scale factor,
    /// using matrix multiplication.
    #[inline]
    pub fn scaled(&self, scale: Vector3) -> Self {
        Transform {
            basis: self.basis.scaled(scale),
            origin: self.origin * scale,
        }
    }

    /// In-place translates the transform by the given offset, relative to
    /// the transform's basis vectors.
    #[inline]
    fn translate_global(&mut self, translation: Vector3) {
        // Note: Godot source uses origin + basis dot translation,
        // but self.translate() uses only origin + translation
        self.origin.x += self.basis.elements[0].dot(translation);
        self.origin.y += self.basis.elements[1].dot(translation);
        self.origin.z += self.basis.elements[2].dot(translation);
    }

    /// Translates the transform by the given offset, relative to
    /// the transform's basis vectors.
    ///
    /// This method will be renamed to `translated` in gdnative 0.11
    #[inline]
    pub fn translated_global(&self, translation: Vector3) -> Self {
        let mut copy = *self;
        copy.translate_global(translation);
        copy
    }

    /// Returns the transform with the basis orthogonal (90 degrees),
    /// and normalized axis vectors.
    #[inline]
    pub fn orthonormalized(&self) -> Self {
        Transform {
            basis: self.basis.orthonormalized(),
            origin: self.origin,
        }
    }

    /// Returns the transform with the basis orthogonal (90 degrees),
    /// but without normalizing the axis vectors.
    #[inline]
    pub fn orthogonalized(&self) -> Self {
        Transform {
            basis: self.basis.orthogonalized(),
            origin: self.origin,
        }
    }

    #[inline]
    pub fn is_equal_approx(&self, other: &Transform) -> bool {
        self.basis.is_equal_approx(&other.basis) && self.origin.is_equal_approx(other.origin)
    }

    /// Interpolates the transform to other Transform by weight amount (on the range of 0.0 to 1.0).
    /// Assuming the two transforms are located on a sphere surface.
    #[inline]
    #[deprecated = "This is the Godot 4 rename of `interpolate_with`. It will be removed in favor of the original Godot 3 naming in a future version."]
    pub fn sphere_interpolate_with(&self, other: &Transform, weight: f32) -> Self {
        self.interpolate_with(other, weight)
    }

    /// Interpolates the transform to other Transform by weight amount (on the range of 0.0 to 1.0).
    /// Assuming the two transforms are located on a sphere surface.
    #[inline]
    pub fn interpolate_with(&self, other: &Transform, weight: f32) -> Self {
        let src_scale = self.basis.scale();
        let src_rot = self.basis.to_quat();
        let src_loc = self.origin;

        let dst_scale = other.basis.scale();
        let dst_rot = other.basis.to_quat();
        let dst_loc = other.origin;

        let new_basis = Basis::from_quat(src_rot.slerp(dst_rot, weight).normalized());
        let new_basis = new_basis.scaled(src_scale.linear_interpolate(dst_scale, weight));
        Transform {
            basis: new_basis,
            origin: src_loc.linear_interpolate(dst_loc, weight),
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
    fn mul(mut self, rhs: Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl MulAssign<Transform> for Transform {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.origin = self.xform(rhs.origin);
        self.basis = self.basis * rhs.basis;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Equivalent GDScript, in case Godot values need to be updated:
    ///
    /// ```gdscript
    /// func test_inputs():
    ///    var basis = Basis(Vector3(37.51756, 20.39467, 49.96816))
    ///    var t = Transform(
    ///        basis.x,
    ///        basis.y,
    ///        basis.z,
    ///        Vector3(0.0, 0.0, 0.0))
    ///    t = t.translated(Vector3(0.5, -1.0, 0.25))
    ///    t = t.scaled(Vector3(0.25, 0.5, 2.0))
    ///
    ///    basis = Basis(Vector3(12.23, 50.46, 93.94))
    ///    var t2 = Transform(
    ///        basis.x,
    ///        basis.y,
    ///        basis.z,
    ///        Vector3(0.0, 0.0, 0.0))
    ///    t2 = t2.translated(Vector3(1.5, -2.0, 1.25))
    ///    t2 = t2.scaled(Vector3(0.5, 0.58, 1.0))
    ///
    ///    return [t, t2]
    /// ```
    fn test_inputs() -> (Transform, Transform) {
        let basis = Basis::from_euler(Vector3::new(37.51756, 20.39467, 49.96816));
        let mut t = Transform::from_basis_origin(
            basis.a(),
            basis.b(),
            basis.c(),
            Vector3::new(0.0, 0.0, 0.0),
        );
        t = t.translated_global(Vector3::new(0.5, -1.0, 0.25));
        t = t.scaled(Vector3::new(0.25, 0.5, 2.0));

        let basis = Basis::from_euler(Vector3::new(12.23, 50.46, 93.94));
        let mut t2 = Transform::from_basis_origin(
            basis.a(),
            basis.b(),
            basis.c(),
            Vector3::new(0.0, 0.0, 0.0),
        );
        t2 = t2.translated_global(Vector3::new(1.5, -2.0, 1.25));
        t2 = t2.scaled(Vector3::new(0.5, 0.58, 1.0));
        // Godot reports:
        // t = 0.019358, -0.041264, 0.24581, -0.144074, 0.470205, 0.090279, -1.908901, -0.594598, 0.050514 - 0.112395, -0.519672, -0.347224
        // t2 = 0.477182, 0.118214, 0.09123, -0.165859, 0.521769, 0.191437, -0.086105, -0.367178, 0.926157 - 0.593383, -1.05303, 1.762894

        (t, t2)
    }

    #[test]
    fn translation_is_sane() {
        let translation = Vector3::new(1.0, 2.0, 3.0);
        let t = Transform::default().translated_local(translation);
        assert!(t.basis.elements[0] == Vector3::new(1.0, 0.0, 0.0));
        assert!(t.basis.elements[1] == Vector3::new(0.0, 1.0, 0.0));
        assert!(t.basis.elements[2] == Vector3::new(0.0, 0.0, 1.0));
        assert!(t.origin == translation);
    }

    #[test]
    fn scale_is_sane() {
        let scale = Vector3::new(1.0, 2.0, 3.0);
        let t = Transform::default().scaled(scale);
        assert!(t.basis.elements[0] == Vector3::new(1.0, 0.0, 0.0));
        assert!(t.basis.elements[1] == Vector3::new(0.0, 2.0, 0.0));
        assert!(t.basis.elements[2] == Vector3::new(0.0, 0.0, 3.0));
        assert!(t.origin == Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn affine_inverse_is_sane() {
        // Godot reports:
        // From 0.019358, -0.041264, 0.24581, -0.144074, 0.470205, 0.090279, -1.908901, -0.594598, 0.050514 - 0.112395, -0.519672, -0.347224
        // To 0.309725, -0.576295, -0.477225, -0.66022, 1.880819, -0.148649, 3.932961, 0.361114, 0.012628 - -0.5, 1, -0.25
        let t = test_inputs().0.affine_inverse();
        let expected = Transform::from_basis_origin(
            Vector3::new(0.309725, -0.66022015, 3.9329607),
            Vector3::new(-0.57629496, 1.8808193, 0.3611141),
            Vector3::new(-0.47722515, -0.14864945, 0.012628445),
            Vector3::new(-0.5, 1.0, -0.25),
        );
        assert!(expected.is_equal_approx(&t))
    }

    #[test]
    fn inverse_is_sane() {
        let t = test_inputs().0.inverse();
        let expected = Transform::from_basis_origin(
            Vector3::new(0.019358, -0.041264, 0.24581),
            Vector3::new(-0.144074, 0.470205, 0.090279),
            Vector3::new(-1.908901, -0.594598, 0.050514),
            Vector3::new(-0.739863, 0.042531, 0.036827),
        );

        println!("TF:  {t:?}");
        assert!(expected.is_equal_approx(&t))
    }

    #[test]
    fn orthonormalization_is_sane() {
        // Godot reports:
        // From 0.019358, -0.041264, 0.24581, -0.144074, 0.470205, 0.090279, -1.908901, -0.594598, 0.050514 - 0.112395, -0.519672, -0.347224
        // To 0.010112, -0.090928, 0.995806, -0.075257, 0.992963, 0.091432, -0.997113, -0.075866, 0.003197 - 0.112395, -0.519672, -0.347224
        let t = test_inputs().0.orthonormalized();
        let expected = Transform::from_basis_origin(
            Vector3::new(0.010111539, -0.0752568, -0.99711293),
            Vector3::new(-0.090927705, 0.9929635, -0.075865656),
            Vector3::new(0.99580616, 0.0914323, 0.0031974507),
            Vector3::new(0.11239518, -0.519672, -0.34722406),
        );
        assert!(expected.is_equal_approx(&t))
    }

    #[test]
    fn spherical_interpolation_is_sane() {
        // Godot reports:
        // t = 0.019358, -0.041264, 0.24581, -0.144074, 0.470205, 0.090279, -1.908901, -0.594598, 0.050514 - 0.112395, -0.519672, -0.347224
        // t2 = 0.477182, 0.118214, 0.09123, -0.165859, 0.521769, 0.191437, -0.086105, -0.367178, 0.926157 - 0.593383, -1.05303, 1.762894
        let (t, t2) = test_inputs();
        let result = t.interpolate_with(&t2, 0.5);
        let expected = Transform::from_basis_origin(
            Vector3::new(0.7279087, -0.19632529, -0.45626357),
            Vector3::new(-0.05011323, 0.65140045, -0.22942543),
            Vector3::new(0.9695858, 0.18105738, 0.33067825),
            Vector3::new(0.3528893, -0.78635097, 0.7078349),
        );
        assert!(expected.is_equal_approx(&result))
    }
}
