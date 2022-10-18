use crate::core_types::{IsEqualApprox, Quat, Vector3};
use crate::globalscope::lerp;
use glam::Mat3;
use std::ops::Mul;

/// A 3x3 matrix, typically used as an orthogonal basis for [`Transform`][crate::core_types::Transform].
///
/// The basis vectors are the column vectors of the matrix, while the [`elements`][Self::elements]
/// field represents the row vectors.
///
/// See also [Basis](https://docs.godotengine.org/en/stable/classes/class_basis.html) in the Godot API doc.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Basis {
    /// Matrix rows. These are **not** the basis vectors!
    ///
    /// This is a transposed view for performance.<br>
    /// To read basis vectors, see [`a()`][Self::a], [`b()`][Self::b], [`c()`][Self::c].<br>
    /// To write them, see [`set_a()`][Self::set_a], [`set_b()`][Self::set_b], [`set_c()`][Self::set_c].
    pub elements: [Vector3; 3],
}

impl Default for Basis {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Basis {
    /// The identity basis. Basis vectors are unit vectors along each axis X, Y and Z.
    ///
    /// Equivalent to calling [`Basis::default()`].
    pub const IDENTITY: Self = Self {
        elements: [
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ],
    };

    /// The basis that will flip something along the **X axis** when used in a transformation.
    pub const FLIP_X: Self = Self::from_diagonal(Vector3::new(-1.0, 1.0, 1.0));

    /// The basis that will flip something along the **Y axis** when used in a transformation.
    pub const FLIP_Y: Self = Self::from_diagonal(Vector3::new(1.0, -1.0, 1.0));

    /// The basis that will flip something along the **Z axis** when used in a transformation.
    pub const FLIP_Z: Self = Self::from_diagonal(Vector3::new(1.0, 1.0, -1.0));

    /// Constructs a basis matrix from 3 linearly independent basis vectors (matrix columns).
    ///
    /// This is the typical way to construct a basis. If you want to fill in the elements one-by-one,
    /// consider using [`Self::from_rows()`].
    #[inline]
    pub const fn from_basis_vectors(a: Vector3, b: Vector3, c: Vector3) -> Self {
        Self {
            elements: [
                Vector3::new(a.x, b.x, c.x),
                Vector3::new(a.y, b.y, c.y),
                Vector3::new(a.z, b.z, c.z),
            ],
        }
    }

    /// Creates a basis from 3 row vectors. These are **not** basis vectors.
    ///
    /// This constructor is mostly useful if you want to write matrix elements
    /// in matrix syntax:
    /// ```
    /// # use gdnative::core_types::{Vector3, Basis};
    /// # let a = Vector3::RIGHT;
    /// # let b = Vector3::UP;
    /// # let c = Vector3::BACK;
    /// let basis = Basis::from_rows(
    ///     Vector3::new(a.x, b.x, c.x),
    ///     Vector3::new(a.y, b.y, c.y),
    ///     Vector3::new(a.z, b.z, c.z),
    /// );
    /// ```
    /// The vectors `a`, `b` and `c` are the basis vectors.<br>
    /// In this particular case, you could also use [`Self::from_basis_vectors(a, b, c)`][Self::from_basis_vectors()] instead.
    #[inline]
    pub const fn from_rows(
        x_components: Vector3,
        y_components: Vector3,
        z_components: Vector3,
    ) -> Self {
        Self {
            elements: [x_components, y_components, z_components],
        }
    }

    /// Creates a diagonal matrix from the given vector.
    ///
    /// Can be used as a basis for a scaling transform.
    /// Each component of `scale` represents the scale factor in the corresponding direction.
    #[inline]
    pub const fn from_diagonal(scale: Vector3) -> Self {
        Self {
            elements: [
                Vector3::new(scale.x, 0.0, 0.0),
                Vector3::new(0.0, scale.y, 0.0),
                Vector3::new(0.0, 0.0, scale.z),
            ],
        }
    }

    /// Creates a rotation matrix from Euler angles.
    ///
    /// The angle vector has XYZ components. However, the angles are applied in YXZ convention:
    /// first **Z**, then **X**, and **Y** last.
    #[inline]
    pub fn from_euler(euler_angles: Vector3) -> Self {
        let mut b = Self::default();
        b.set_euler_yxz(euler_angles);
        b
    }

    #[inline]
    fn set_euler_yxz(&mut self, euler: Vector3) {
        let c = euler.x.cos();
        let s = euler.x.sin();
        let xmat = Self::from_rows(
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, c, -s),
            Vector3::new(0.0, s, c),
        );
        let c = euler.y.cos();
        let s = euler.y.sin();
        let ymat = Self::from_rows(
            Vector3::new(c, 0.0, s),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(-s, 0.0, c),
        );

        let c = euler.z.cos();
        let s = euler.z.sin();
        let zmat = Self::from_rows(
            Vector3::new(c, -s, 0.0),
            Vector3::new(s, c, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        );

        *self = ymat * xmat * zmat;
    }

    /// Constructs a pure rotation basis matrix from the given quaternion.
    #[inline]
    pub fn from_quat(quat: Quat) -> Self {
        let basis = Mat3::from_quat(quat.glam()).to_cols_array_2d();

        Self::from_rows(
            Vector3::new(basis[0][0], basis[1][0], basis[2][0]),
            Vector3::new(basis[0][1], basis[1][1], basis[2][1]),
            Vector3::new(basis[0][2], basis[1][2], basis[2][2]),
        )
    }

    /// Rotation matrix from axis and angle.
    ///
    /// See <https://en.wikipedia.org/wiki/Rotation_matrix#Rotation_matrix_from_axis_and_angle>
    ///
    /// # Panics
    ///
    /// If `axis` is not normalized.
    #[inline]
    pub fn from_axis_angle(axis: Vector3, phi: f32) -> Self {
        assert!(
            axis.length().is_equal_approx(1.0),
            "The axis Vector3 must be normalized."
        );

        let mut basis = Self::default();
        let [x, y, z] = &mut basis.elements;

        let axis_sq = Vector3::new(axis.x * axis.x, axis.y * axis.y, axis.z * axis.z);
        let cosine = phi.cos();

        x.x = axis_sq.x + cosine * (1.0 - axis_sq.x);
        y.y = axis_sq.y + cosine * (1.0 - axis_sq.y);
        z.z = axis_sq.z + cosine * (1.0 - axis_sq.z);

        let sine = phi.sin();
        let t = 1.0 - cosine;

        let mut xyzt = axis.x * axis.y * t;
        let mut zyxs = axis.z * sine;
        x.y = xyzt - zyxs;
        y.x = xyzt + zyxs;

        xyzt = axis.x * axis.z * t;
        zyxs = axis.y * sine;
        x.z = xyzt + zyxs;
        z.x = xyzt - zyxs;

        xyzt = axis.y * axis.z * t;
        zyxs = axis.x * sine;
        y.z = xyzt - zyxs;
        z.y = xyzt + zyxs;

        basis
    }

    /// Inverts the matrix.
    ///
    /// # Panics
    ///
    /// If the determinant of `self` is zero.
    #[inline]
    fn invert(&mut self) {
        let [x, y, z] = self.elements;

        let co = [
            y.y * z.z - y.z * z.y,
            y.z * z.x - y.x * z.z,
            y.x * z.y - y.y * z.x,
        ];

        let det: f32 = x.x * co[0] + x.y * co[1] + x.z * co[2];
        assert!(!det.is_equal_approx(0.0), "Determinant was zero");

        let s: f32 = 1.0 / det;

        self.set_a(Vector3::new(co[0] * s, co[1] * s, co[2] * s));
        self.set_b(Vector3::new(
            (x.z * z.y - x.y * z.z) * s,
            (x.x * z.z - x.z * z.x) * s,
            (x.y * z.x - x.x * z.y) * s,
        ));
        self.set_c(Vector3::new(
            (x.y * y.z - x.z * y.y) * s,
            (x.z * y.x - x.x * y.z) * s,
            (x.x * y.y - x.y * y.x) * s,
        ));
    }

    /// Returns the inverse of the matrix.
    ///
    /// # Panics
    ///
    /// If the determinant of `self` is zero.
    #[inline]
    pub fn inverse(&self) -> Self {
        let mut copy = *self;
        copy.invert();
        copy
    }

    #[inline]
    #[deprecated = "Use `inverse` instead."]
    pub fn inverted(&self) -> Self {
        self.inverse()
    }

    /// Returns linear interpolation on a sphere between two basis by weight amount (on the range of 0.0 to 1.0).
    #[inline]
    pub fn slerp(&self, other: &Basis, weight: f32) -> Self {
        let from = self.to_quat();
        let to = other.to_quat();
        let mut result = Basis::from_quat(from.slerp(to, weight));

        for i in 0..3 {
            result.elements[i] *= lerp(
                self.elements[i].length()..=other.elements[i].length(),
                weight,
            );
        }

        result
    }

    /// Returns linear interpolation between two basis by weight amount (on the range of 0.0 to 1.0).
    #[inline]
    pub fn lerp(&self, other: &Basis, weight: f32) -> Self {
        // this is how godot is doing it at https://github.com/godotengine/godot/blob/master/core/math/basis.cpp#L964
        // but Godot engine output for me differs than godot-rust
        let a = self.elements[0].linear_interpolate(other.elements[0], weight);
        let b = self.elements[1].linear_interpolate(other.elements[1], weight);
        let c = self.elements[2].linear_interpolate(other.elements[2], weight);
        Basis::from_rows(a, b, c)
    }

    /// Transposes the matrix.
    #[inline]
    fn transpose(&mut self) {
        std::mem::swap(&mut self.elements[0].y, &mut self.elements[1].x);
        std::mem::swap(&mut self.elements[0].z, &mut self.elements[2].x);
        std::mem::swap(&mut self.elements[1].z, &mut self.elements[2].y);
    }

    /// Returns the transposed version of the matrix.
    #[inline]
    pub fn transposed(&self) -> Self {
        let mut copy = *self;
        copy.transpose();
        copy
    }

    /// Returns the determinant of the matrix.
    #[inline]
    pub fn determinant(&self) -> f32 {
        let [x, y, z] = &self.elements;
        x.x * (y.y * z.z - z.y * y.z) // x
            - y.x * (x.y * z.z - z.y * x.z) // y
            + z.x * (x.y * y.z - y.y * x.z) // z
    }

    /// Orthonormalizes the matrix.
    ///
    /// Performs a [Gram-Schmidt orthonormalization](https://en.wikipedia.org/wiki/Gram-Schmidt_process) on the basis of the matrix.
    /// This can be useful to call from time to time to avoid rounding errors for orthogonal matrices.
    ///
    /// # Panics
    ///
    /// If the determinant of `self` is zero.
    #[inline]
    fn orthonormalize(&mut self) {
        assert!(
            !self.determinant().is_equal_approx(0.0),
            "Determinant should not be zero."
        );

        // Gram-Schmidt Process
        let mut x = self.a();
        let mut y = self.b();
        let mut z = self.c();

        x = x.normalized();
        y = y - x * (x.dot(y));
        y = y.normalized();
        z = z - x * (x.dot(z)) - y * (y.dot(z));
        z = z.normalized();

        self.set_a(x);
        self.set_b(y);
        self.set_c(z);
    }

    #[inline]
    fn is_orthogonal(&self) -> bool {
        let m = (*self) * self.transposed();
        m.is_equal_approx(&Self::IDENTITY)
    }

    #[inline]
    fn orthogonalize(&mut self) {
        let scale = self.scale();
        self.orthonormalize();
        self.scale_local(scale);
    }

    #[inline]
    pub fn orthogonalized(&self) -> Self {
        let mut copy = *self;
        copy.orthogonalize();
        copy
    }

    /// Returns an orthonormalized version of the matrix: 3 orthogonal basis vectors of unit length.
    #[inline]
    pub fn orthonormalized(&self) -> Self {
        let mut copy = *self;
        copy.orthonormalize();
        copy
    }

    /// Returns `true` if `self` and `other` are approximately equal.
    #[inline]
    pub fn is_equal_approx(&self, other: &Basis) -> bool {
        self.elements[0].is_equal_approx(other.elements[0])
            && self.elements[1].is_equal_approx(other.elements[1])
            && self.elements[2].is_equal_approx(other.elements[2])
    }

    /// Multiplies the matrix from left with the rotation matrix: M -> R·M
    ///
    /// The main use of `Basis` is as a `Transform.basis`, which is used as the transformation matrix
    /// of the 3D object. `rotated()` here refers to rotation of the object (which is `R * self`), not the matrix itself.
    #[inline]
    pub fn rotated(&self, axis: Vector3, phi: f32) -> Self {
        let mut copy = *self;
        copy.rotate(axis, phi);
        copy
    }

    /// Rotates the matrix.
    ///
    /// If object rotation is needed, see [`Basis::rotated()`]
    #[inline]
    fn rotate(&mut self, axis: Vector3, phi: f32) {
        let rot = Self::from_axis_angle(axis, phi);
        *self = rot * *self;
    }

    /// Returns true if this basis represents a rotation matrix (orthogonal and no scaling).
    #[inline]
    fn is_rotation(&self) -> bool {
        let det = self.determinant();
        det.is_equal_approx(1.0) && self.is_orthogonal()
    }

    /// Returns the scale of the matrix.
    #[inline]
    pub fn scale(&self) -> Vector3 {
        let det = self.determinant();
        let det_sign = if det < 0.0 { -1.0 } else { 1.0 };

        Vector3::new(self.a().length(), self.b().length(), self.c().length()) * det_sign
    }

    /// Introduce an additional scaling specified by the given 3D scaling factor.
    #[inline]
    pub fn scaled(&self, scale: Vector3) -> Self {
        let mut copy = *self;
        copy.scale_self(scale);
        copy
    }

    /// In-place basis scaling in object-local coordinate system
    #[inline]
    fn scale_local(&mut self, scale: Vector3) {
        *self = self.scaled_local(scale);
    }

    /// Returns basis scaled in object-local coordinate system
    #[inline]
    fn scaled_local(&self, scale: Vector3) -> Self {
        (*self) * Basis::from_diagonal(scale)
    }

    /// Multiplies the matrix from left with the scaling matrix: M -> S·M
    ///
    /// See the comment for [Basis::rotated](#method.rotated) for further explanation.
    #[inline]
    fn scale_self(&mut self, s: Vector3) {
        self.elements[0] *= s.x;
        self.elements[1] *= s.y;
        self.elements[2] *= s.z;
    }

    /// Converts matrix into a quaternion.
    ///
    /// Quaternions are frequently used in 3D graphics, because they enable easy and cheap interpolation.
    /// However, they are less human-readable. For Euler angles, see [`Basis::to_euler()`].
    ///
    /// # Panics
    ///
    /// If `self` is not normalized.
    #[inline]
    pub fn to_quat(&self) -> Quat {
        // Assumes that the matrix can be decomposed into a proper rotation and scaling matrix as M = R.S,
        // and returns the Euler angles corresponding to the rotation part, complementing get_scale().
        // See the comment in get_scale() for further information.
        let mut matrix = self.orthonormalized();
        let det = matrix.determinant();
        if det < 0.0 {
            // Ensure that the determinant is 1, such that result is a proper rotation matrix which can be represented by Euler angles.
            matrix.scale_self(Vector3::new(-1.0, -1.0, -1.0));
        }

        assert!(matrix.is_rotation(), "Basis must be normalized in order to be casted to a Quaternion. Use to_quat() or call orthonormalized() instead.");

        // Allow getting a quaternion from an unnormalized transform
        let trace = matrix.elements[0].x + matrix.elements[1].y + matrix.elements[2].z;
        let mut temp = [0_f32; 4];

        if trace > 0.0 {
            let mut s = (trace + 1.0).sqrt();
            temp[3] = s * 0.5;
            s = 0.5 / s;

            temp[0] = (matrix.elements[2].y - matrix.elements[1].z) * s;
            temp[1] = (matrix.elements[0].z - matrix.elements[2].x) * s;
            temp[2] = (matrix.elements[1].x - matrix.elements[0].y) * s;
        } else {
            let i = if matrix.elements[0].x < matrix.elements[1].y {
                if matrix.elements[1].y < matrix.elements[2].z {
                    2
                } else {
                    1
                }
            } else if matrix.elements[0].x < matrix.elements[2].z {
                2
            } else {
                0
            };

            let j = (i + 1) % 3;
            let k = (i + 2) % 3;

            let elements_arr: [[f32; 3]; 3] = [
                *matrix.elements[0].as_ref(),
                *matrix.elements[1].as_ref(),
                *matrix.elements[2].as_ref(),
            ];

            let mut s = (elements_arr[i][i] - elements_arr[j][j] - elements_arr[k][k] + 1.0).sqrt();
            temp[i] = s * 0.5;
            s = 0.5 / s;

            temp[3] = (elements_arr[k][j] - elements_arr[j][k]) * s;
            temp[j] = (elements_arr[j][i] + elements_arr[i][j]) * s;
            temp[k] = (elements_arr[k][i] + elements_arr[i][k]) * s;
        }

        let [a, b, c, r] = temp;
        Quat::new(a, b, c, r)
    }

    /// Returns the `Basis`’s rotation in the form of Euler angles.
    ///
    /// In the YXZ convention: first **Z**, then **X**, and **Y** last.
    ///
    /// The returned `Vector3` contains the rotation angles in the format (**X** angle, **Y** angle, **Z** angle).
    ///
    /// See [`Basis::to_quat`](#method.to_quat) if you need a quaternion instead.
    #[inline]
    pub fn to_euler(&self) -> Vector3 {
        let mut euler = Vector3::ZERO;

        let m12 = self.elements[1].z;
        if m12 < 1.0 {
            if m12 > -1.0 {
                // is this a pure X rotation?
                if self.elements[1].x.is_equal_approx(0.0)
                    && self.elements[0].y.is_equal_approx(0.0)
                    && self.elements[0].z.is_equal_approx(0.0)
                    && self.elements[2].x.is_equal_approx(0.0)
                    && self.elements[0].x.is_equal_approx(1.0)
                {
                    // return the simplest form (human friendlier in editor and scripts)
                    euler.x = (-m12).atan2(self.elements[1].y);
                    euler.y = 0.0;
                    euler.z = 0.0;
                } else {
                    euler.x = (-m12).asin();
                    euler.y = self.elements[0].z.atan2(self.elements[2].z);
                    euler.z = self.elements[1].x.atan2(self.elements[1].y);
                }
            } else {
                // m12 == -1
                euler.x = core::f32::consts::PI * 0.5;
                euler.y = -(-self.elements[0].y).atan2(self.elements[0].x);
                euler.z = 0.0;
            }
        } else {
            // m12 == 1
            euler.x = -core::f32::consts::PI * 0.5;
            euler.y = -(-self.elements[0].y).atan2(self.elements[0].x);
            euler.z = 0.0;
        }

        euler
    }

    /// Returns a vector transformed (multiplied) by the matrix.
    #[inline]
    pub fn xform(&self, v: Vector3) -> Vector3 {
        Vector3::new(
            self.elements[0].dot(v),
            self.elements[1].dot(v),
            self.elements[2].dot(v),
        )
    }

    /// Returns a vector transformed (multiplied) by the transposed matrix.
    ///
    /// Note: This results in a multiplication by the inverse of the matrix only if it represents a rotation-reflection.
    #[inline]
    pub fn xform_inv(&self, v: Vector3) -> Vector3 {
        Vector3::new(self.a().dot(v), self.b().dot(v), self.c().dot(v))
    }

    /// Transposed dot product with the **X basis vector** of the matrix.
    #[inline]
    pub(crate) fn tdotx(&self, v: Vector3) -> f32 {
        self.a().dot(v)
    }

    /// Transposed dot product with the **Y basis vector** of the matrix.
    #[inline]
    pub(crate) fn tdoty(&self, v: Vector3) -> f32 {
        self.b().dot(v)
    }

    /// Transposed dot product with the **Z basis vector** of the matrix.
    #[inline]
    pub(crate) fn tdotz(&self, v: Vector3) -> f32 {
        self.c().dot(v)
    }

    /// Get the **1st basis vector** (first column vector of the matrix).
    #[inline]
    pub fn a(&self) -> Vector3 {
        Vector3::new(self.elements[0].x, self.elements[1].x, self.elements[2].x)
    }

    /// Set the **1st basis vector** (first column vector of the matrix).
    #[inline]
    pub fn set_a(&mut self, v: Vector3) {
        self.elements[0].x = v.x;
        self.elements[1].x = v.y;
        self.elements[2].x = v.z;
    }

    /// Get the **2nd basis vector** (second column vector of the matrix).
    #[inline]
    pub fn b(&self) -> Vector3 {
        Vector3::new(self.elements[0].y, self.elements[1].y, self.elements[2].y)
    }

    /// Set the **2nd basis vector** (second column vector of the matrix).
    #[inline]
    pub fn set_b(&mut self, v: Vector3) {
        self.elements[0].y = v.x;
        self.elements[1].y = v.y;
        self.elements[2].y = v.z;
    }

    /// Get the **3rd basis vector** (third column vector of the matrix).
    #[inline]
    pub fn c(&self) -> Vector3 {
        Vector3::new(self.elements[0].z, self.elements[1].z, self.elements[2].z)
    }

    /// Set the **3rd basis vector** (third column vector of the matrix).
    #[inline]
    pub fn set_c(&mut self, v: Vector3) {
        self.elements[0].z = v.x;
        self.elements[1].z = v.y;
        self.elements[2].z = v.z;
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_basis {
        unsafe { std::mem::transmute::<*const Basis, *const sys::godot_basis>(self as *const _) }
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(c: sys::godot_basis) -> Self {
        unsafe { std::mem::transmute::<sys::godot_basis, Self>(c) }
    }
}

impl Mul<Basis> for Basis {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Basis::from_rows(
            Vector3::new(
                rhs.tdotx(self.elements[0]),
                rhs.tdoty(self.elements[0]),
                rhs.tdotz(self.elements[0]),
            ),
            Vector3::new(
                rhs.tdotx(self.elements[1]),
                rhs.tdoty(self.elements[1]),
                rhs.tdotz(self.elements[1]),
            ),
            Vector3::new(
                rhs.tdotx(self.elements[2]),
                rhs.tdoty(self.elements[2]),
                rhs.tdotz(self.elements[2]),
            ),
        )
    }
}

impl Mul<Vector3> for Basis {
    type Output = Vector3;

    #[inline]
    fn mul(self, rhs: Self::Output) -> Self::Output {
        self.xform(rhs)
    }
}

#[cfg(test)]
#[allow(clippy::unreadable_literal)]
mod tests {
    use super::*;

    #[test]
    fn transposed_dot_is_sane() {
        let basis = Basis {
            elements: [
                Vector3::new(1.0, 2.0, 3.0),
                Vector3::new(2.0, 3.0, 4.0),
                Vector3::new(3.0, 4.0, 5.0),
            ],
        };

        let vector = Vector3::new(4.0, 5.0, 6.0);

        assert!((basis.tdotx(vector) - 32.0).abs() < std::f32::EPSILON);
        assert!((basis.tdoty(vector) - 47.0).abs() < std::f32::EPSILON);
        assert!((basis.tdotz(vector) - 62.0).abs() < std::f32::EPSILON);
    }

    #[test]
    fn retrieval_is_sane() {
        let basis = Basis {
            elements: [
                Vector3::new(1.0, 2.0, 3.0),
                Vector3::new(4.0, 5.0, 6.0),
                Vector3::new(7.0, 8.0, 9.0),
            ],
        };

        assert!(basis.a() == Vector3::new(1.0, 4.0, 7.0));
        assert!(basis.b() == Vector3::new(2.0, 5.0, 8.0));
        assert!(basis.c() == Vector3::new(3.0, 6.0, 9.0));
    }

    #[test]
    fn set_is_sane() {
        let mut basis = Basis {
            elements: [Vector3::ZERO, Vector3::ZERO, Vector3::ZERO],
        };

        basis.set_a(Vector3::new(1.0, 4.0, 7.0));
        basis.set_b(Vector3::new(2.0, 5.0, 8.0));
        basis.set_c(Vector3::new(3.0, 6.0, 9.0));

        assert!(basis.elements[0] == Vector3::new(1.0, 2.0, 3.0));
        assert!(basis.elements[1] == Vector3::new(4.0, 5.0, 6.0));
        assert!(basis.elements[2] == Vector3::new(7.0, 8.0, 9.0));
    }

    fn test_inputs() -> (Basis, Basis) {
        let v = Vector3::new(37.51756, 20.39467, 49.96816);
        let vn = v.normalized();
        let b = Basis::from_euler(v);
        let bn = Basis::from_euler(vn);
        (b, bn)
    }

    #[test]
    fn determinant() {
        let (b, _bn) = test_inputs();

        assert!(
            b.determinant().is_equal_approx(1.0),
            "Determinant should be 1.0"
        );
    }

    #[test]
    fn euler() {
        let (_b, bn) = test_inputs();

        assert!(Vector3::new(0.57079, 0.310283, 0.760213).is_equal_approx(bn.to_euler()));
    }

    #[test]
    fn orthonormalized() {
        let (b, _bn) = test_inputs();

        let expected = Basis::from_rows(
            Vector3::new(0.077431, -0.165055, 0.98324),
            Vector3::new(-0.288147, 0.94041, 0.180557),
            Vector3::new(-0.95445, -0.297299, 0.025257),
        );
        assert!(expected.is_equal_approx(&b.orthonormalized()));
    }

    #[test]
    fn scaled() {
        let (b, _bn) = test_inputs();

        let expected = Basis::from_rows(
            Vector3::new(0.052484, -0.111876, 0.666453),
            Vector3::new(0.012407, -0.040492, -0.007774),
            Vector3::new(-0.682131, -0.212475, 0.018051),
        );
        assert!(expected.is_equal_approx(&b.scaled(Vector3::new(0.677813, -0.043058, 0.714685))));
    }

    #[test]
    fn rotated() {
        let (b, _bn) = test_inputs();

        let r = Vector3::new(-50.167156, 60.67781, -70.04305).normalized();
        let expected = Basis::from_rows(
            Vector3::new(-0.676245, 0.113805, 0.727833),
            Vector3::new(-0.467094, 0.697765, -0.54309),
            Vector3::new(-0.569663, -0.707229, -0.418703),
        );
        assert!(expected.is_equal_approx(&b.rotated(r, 1.0)));
    }

    #[test]
    fn to_quat() {
        let (b, _bn) = test_inputs();

        assert!(Quat::new(-0.167156, 0.677813, -0.043058, 0.714685).is_equal_approx(b.to_quat()));
    }

    #[test]
    fn scale() {
        let (b, _bn) = test_inputs();

        assert!(Vector3::new(1.0, 1.0, 1.0).is_equal_approx(b.scale()));
    }

    #[test]
    fn approx_eq() {
        let (b, _bn) = test_inputs();
        assert!(!b.is_equal_approx(&Basis::from_euler(Vector3::new(37.517, 20.394, 49.968))));
    }

    #[test]
    fn transposed() {
        let (b, _bn) = test_inputs();
        let expected = Basis::from_rows(
            Vector3::new(0.077431, -0.288147, -0.95445),
            Vector3::new(-0.165055, 0.94041, -0.297299),
            Vector3::new(0.98324, 0.180557, 0.025257),
        );
        assert!(expected.is_equal_approx(&b.transposed()));
    }

    #[test]
    fn xform() {
        let (b, _bn) = test_inputs();

        assert!(Vector3::new(-0.273471, 0.478102, -0.690386)
            .is_equal_approx(b.xform(Vector3::new(0.5, 0.7, -0.2))));
    }

    #[test]
    fn xform_inv() {
        let (b, _bn) = test_inputs();

        assert!(Vector3::new(-0.884898, -0.460316, 0.071165)
            .is_equal_approx(b.xform_inv(Vector3::new(0.077431, -0.165055, 0.98324))));
    }

    #[test]
    fn inverse() {
        let (b, _bn) = test_inputs();

        let expected = Basis::from_rows(
            Vector3::new(0.077431, -0.288147, -0.95445),
            Vector3::new(-0.165055, 0.94041, -0.297299),
            Vector3::new(0.98324, 0.180557, 0.025257),
        );
        assert!(expected.is_equal_approx(&b.inverse()));
    }
}
