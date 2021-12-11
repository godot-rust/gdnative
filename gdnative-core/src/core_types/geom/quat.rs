use crate::core_types::{Basis, IsEqualApprox, Vector3, CMP_EPSILON};
use glam::EulerRot;
use std::ops::{Mul, Neg};

/// Quaternion, used to represent 3D rotations.
///
/// Quaternions need to be [normalized][Self::normalized()] before all operations.
///
/// See also [Quat](https://docs.godotengine.org/en/stable/classes/class_quat.html) in the Godot API doc.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Helper methods for `Quat`.
///
/// See the official [`Godot documentation`](https://docs.godotengine.org/en/stable/classes/class_quat.html).
impl Quat {
    /// The identity quaternion, representing no rotation. Equivalent to an identity [`Basis`] matrix.
    /// If a vector is transformed by an identity quaternion, it will not change.
    pub const IDENTITY: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// Constructs a quaternion defined by the given values.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Constructs a quaternion from the given [Basis]
    #[inline]
    pub fn from_basis(basis: &Basis) -> Self {
        basis.to_quat()
    }

    /// Constructs a quaternion that will perform a rotation specified by Euler angles (in the YXZ
    /// convention: when decomposing, first Z, then X, and Y last), given in the vector format as
    /// (X angle, Y angle, Z angle).
    #[inline]
    pub fn from_euler(euler: Vector3) -> Self {
        Self::gd(glam::Quat::from_euler(
            EulerRot::YXZ,
            euler.y,
            euler.x,
            euler.z,
        ))
    }

    /// Constructs a quaternion that will rotate around the given axis by the specified angle. The
    /// axis must be a normalized vector.
    #[inline]
    pub fn from_axis_angle(axis: Vector3, angle: f32) -> Self {
        debug_assert!(axis.is_normalized(), "Axis is not normalized");
        Self::gd(glam::Quat::from_axis_angle(axis.glam().into(), angle))
    }

    /// Performs a cubic spherical interpolation between quaternions `pre_a`, this quaternion, `b`,
    /// and `post_b`, by the given amount `t`.
    #[inline]
    pub fn cubic_slerp(self, b: Self, pre_a: Self, post_b: Self, t: f32) -> Self {
        let t2 = (1.0 - t) * t * 2.0;
        let sp = self.slerp(b, t);
        let sq = pre_a.slerpni(post_b, t);
        sp.slerpni(sq, t2)
    }

    /// Returns the dot product of two quaternions.
    #[inline]
    pub fn dot(self, b: Self) -> f32 {
        self.glam().dot(b.glam())
    }

    /// Returns Euler angles (in the YXZ convention: when decomposing, first Z, then X, and Y last)
    /// corresponding to the rotation represented by the unit quaternion. Returned vector contains
    /// the rotation angles in the format (X angle, Y angle, Z angle).
    #[inline]
    pub fn to_euler(self) -> Vector3 {
        Basis::from_quat(self).to_euler()
    }

    /// Returns the inverse of the quaternion.
    #[inline]
    pub fn inverse(self) -> Self {
        Self::gd(self.glam().inverse())
    }

    /// Returns `true` if this quaternion and `quat` are approximately equal, by running
    /// `is_equal_approx` on each component
    #[inline]
    pub fn is_equal_approx(self, to: Self) -> bool {
        self.x.is_equal_approx(to.x)
            && self.y.is_equal_approx(to.y)
            && self.z.is_equal_approx(to.z)
            && self.w.is_equal_approx(to.w)
    }

    /// Returns whether the quaternion is normalized or not.
    #[inline]
    pub fn is_normalized(self) -> bool {
        self.glam().is_normalized()
    }

    /// Returns the length of the quaternion.
    #[inline]
    pub fn length(self) -> f32 {
        self.glam().length()
    }

    /// Returns the length of the quaternion, squared.
    #[inline]
    pub fn length_squared(self) -> f32 {
        self.glam().length_squared()
    }

    /// Returns a copy of the quaternion, normalized to unit length.
    ///
    /// Normalization is necessary before transforming vectors through `xform()` or `*`.
    #[inline]
    pub fn normalized(self) -> Self {
        Self::gd(self.glam().normalize())
    }

    /// Returns the result of the spherical linear interpolation between this quaternion and to by
    /// amount weight.
    ///
    /// **Note:** Both quaternions must be normalized.
    #[inline]
    pub fn slerp(self, b: Self, t: f32) -> Self {
        debug_assert!(self.is_normalized(), "Quaternion `self` is not normalized");
        debug_assert!(b.is_normalized(), "Quaternion `b` is not normalized");

        // Copied from Godot's Quat::slerp as glam::lerp version diverges too much

        // calc cosine
        let cos = self.dot(b);

        // adjust signs (if necessary)
        let (cos, b) = if cos < 0.0 { (-cos, -b) } else { (cos, b) };

        // calculate coefficients
        let scale = if 1.0 - cos > CMP_EPSILON as f32 {
            // standard case (slerp)
            let omega = cos.acos();
            let sin = omega.sin();
            (((1.0 - t) * omega).sin() / sin, (t * omega).sin() / sin)
        } else {
            // "from" and "to" quaternions are very close
            //  ... so we can do a linear interpolation
            (1.0 - t, t)
        };

        // calculate final values
        Self::new(
            scale.0 * self.x + scale.1 * b.x,
            scale.0 * self.y + scale.1 * b.y,
            scale.0 * self.z + scale.1 * b.z,
            scale.0 * self.w + scale.1 * b.w,
        )
    }

    /// Returns the result of the spherical linear interpolation between this quaternion and `t` by
    /// amount `t`, but without checking if the rotation path is not bigger than 90 degrees.
    #[inline]
    pub fn slerpni(self, b: Self, t: f32) -> Self {
        debug_assert!(self.is_normalized(), "Quaternion `self` is not normalized");
        debug_assert!(b.is_normalized(), "Quaternion `b` is not normalized");

        // Copied from Godot's Quat::slerpni as glam::slerp version diverges too much

        let dot = self.dot(b);
        if dot.abs() > 0.9999 {
            self
        } else {
            let theta = dot.acos();
            let sin_t = 1.0 / theta.sin();
            let new_f = (t * theta).sin() * sin_t;
            let inv_f = ((1.0 - t) * theta).sin() * sin_t;
            Self::new(
                inv_f * self.x + new_f * b.x,
                inv_f * self.y + new_f * b.y,
                inv_f * self.z + new_f * b.z,
                inv_f * self.w + new_f * b.w,
            )
        }
    }

    /// Returns a vector transformed (multiplied) by this quaternion. This is the same as `mul`
    ///
    /// **Note:** The quaternion must be normalized.
    #[inline]
    pub fn xform(self, v: Vector3) -> Vector3 {
        self * v
    }

    #[inline]
    pub(super) fn gd(quat: glam::Quat) -> Self {
        Self::new(quat.x, quat.y, quat.z, quat.w)
    }

    #[inline]
    pub(super) fn glam(self) -> glam::Quat {
        glam::Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}

impl Mul<Vector3> for Quat {
    type Output = Vector3;

    #[inline]
    /// Returns a vector transformed (multiplied) by this quaternion. This is the same as `xform`
    ///
    /// **Note:** The quaternion must be normalized.
    fn mul(self, with: Vector3) -> Vector3 {
        debug_assert!(self.is_normalized(), "Quaternion is not normalized");
        Vector3::gd(self.glam() * with.glam())
    }
}

impl Mul<Self> for Quat {
    type Output = Self;

    #[inline]
    /// Returns another quaternion transformed (multiplied) by this quaternion.
    fn mul(self, with: Self) -> Self {
        Self::gd(self.glam() * with.glam())
    }
}

impl Neg for Quat {
    type Output = Quat;

    #[inline]
    fn neg(self) -> Self {
        Self::gd(-self.glam())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_euler() {
        let euler = Vector3::new(0.25, 5.24, 3.0);
        let expect = Quat::new(0.485489, 0.142796, -0.862501, 0.001113);
        assert!(Quat::from_euler(euler).is_equal_approx(expect));
    }

    #[test]
    fn to_basis() {
        let quat = Quat::new(0.485489, 0.142796, -0.862501, 0.001113);
        let expect = Basis::from_rows(
            Vector3::new(-0.528598, 0.140572, -0.837152),
            Vector3::new(0.136732, -0.959216, -0.247404),
            Vector3::new(-0.837788, -0.245243, 0.487819),
        );
        let basis = Basis::from_quat(quat);
        assert!(basis.is_equal_approx(&expect));
    }

    #[test]
    fn to_euler() {
        let quat = Quat::new(0.485489, 0.142796, -0.862501, 0.001113);
        let expect = Vector3::new(0.25, -1.043185, 3.00001);
        assert!(quat.to_euler().is_equal_approx(expect));
    }

    #[test]
    fn mul_vec() {
        let quat = Quat::new(0.485489, 0.142796, -0.862501, 0.001113);
        let vec = Vector3::new(2.2, 0.8, 1.65);
        let expect = Vector3::new(-2.43176, -0.874777, -1.234427);
        assert!(expect.is_equal_approx(quat * vec));
    }

    #[test]
    fn mul_quat() {
        let a = Quat::new(-0.635115, -0.705592, 0.314052, 0.011812);
        let b = Quat::new(0.485489, 0.142796, -0.862501, 0.001113);
        let e = Quat::new(0.568756, -0.394417, 0.242027, 0.67998);
        assert!(e.is_equal_approx(a * b));
    }

    #[test]
    fn slerp() {
        let q = Quat::new(-0.635115, -0.705592, 0.314052, 0.011812);
        let p = Quat::new(0.485489, 0.142796, -0.862501, 0.001113);
        let t = 0.2;
        let e = Quat::new(-0.638517, -0.620742, 0.454844, 0.009609);
        assert!(e.is_equal_approx(q.slerp(p, t)));
    }

    #[test]
    fn slerpni() {
        let q = Quat::new(-0.635115, -0.705592, 0.314052, 0.011812);
        let p = Quat::new(0.485489, 0.142796, -0.862501, 0.001113);
        let t = 0.2;
        let e = Quat::new(-0.535331, -0.836627, -0.114954, 0.016143);
        assert!(e.is_equal_approx(q.slerpni(p, t)));
    }

    #[test]
    fn cubic_slerp() {
        let a = Quat::new(-0.635115, -0.705592, 0.314052, 0.011812);
        let b = Quat::new(0.485489, 0.142796, -0.862501, 0.001113);
        let c = Quat::new(-0.666276, 0.03859, 0.083527, -0.740007);
        let d = Quat::new(-0.856633, -0.430228, -0.284017, 0.020464);
        let t = 0.2;
        let e = Quat::new(-0.768253, -0.490687, 0.341836, -0.22839);
        assert!(e.is_equal_approx(a.cubic_slerp(b, c, d, t)));
    }
}
